//use std::collections::{HashMap};
//use std::collections::hash_map::Entry;
//use std::rc::Rc;
use rand::prelude::*;
use rand::{ChaChaRng, Rng, SeedableRng};
//use rpds::map::hash_trie_map::HashTrieMap;
use arrayvec::ArrayVec;
use smallvec::SmallVec;
use std::cmp::{max, min};

use super::blueprint::*;
use super::simplified::*;

#[derive(Clone, Debug)]
pub struct RoutingMap {
  grid: Grid<bool>,
}

impl RoutingMap {
  pub fn new(bounds: Rectangle) -> RoutingMap {
    RoutingMap {
      grid: Grid::new(bounds),
    }
  }
}
/*
fn clone_with_room<T: Clone>(source: &[T]) -> Vec<T> {
  if source.is_empty() {
    return Vec::new();
  } // because allocating with capacity one will never save an allocation, and might be unnecessary
  let mut result = Vec::with_capacity(source.len() + 1);
  result.extend_from_slice(source);
  result
}*/

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RouteOutput {
  Conveyor(DirectedEdge),
  InsertInto(Coordinates),
  InsertFrom(Coordinates),
}

impl RoutingMap {
  pub fn out_of_bounds(&self, coordinates: [i32; 2]) -> bool {
    !self.grid.bounds().contains(coordinates)
  }
  pub fn obstructed(&self, coordinates: [i32; 2]) -> bool {
    self.grid.get(coordinates).cloned().unwrap_or(true)
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RouteDestination {
  Conveyor(DirectedEdge),
  Assembler(Assembler),
}

impl RouteOutput {
  pub fn satisfies(&self, destination: &RouteDestination) -> bool {
    match (self, destination) {
      (RouteOutput::Conveyor(output), RouteDestination::Conveyor(destination)) => {
        output == destination
      }
      (RouteOutput::InsertInto(output), RouteDestination::Assembler(destination)) => {
        destination.shape().contains(*output)
      }
      _ => false,
    }
  }
}

pub struct RouteSearchParameters<'a> {
  pub map: &'a RoutingMap,
  //pub previous_version: Option <& 'a Route>,
  pub search_map: &'a mut RouteSearchMap,
  pub overlap_penalty: usize,
  pub max_cost: usize,
  pub source: DirectedEdge,
  pub destinations: &'a [RouteDestination],
  pub backwards: bool,
}
pub struct RouteSearch<'a> {
  parameters: RouteSearchParameters<'a>,
  committed_objects: Vec<Object>,
  frontiers: Vec<Vec<RouteOutput>>,
  current_frontier: usize,
  destinations_satisfied: Vec<bool>,
  failed: bool,
}

/*
#[derive(Clone, Debug)]
pub struct RouteHistoryMap {
  grid: Grid<RouteHistoryMapTile>,
}

#[derive(Clone, Debug, Default)]
pub struct RouteHistoryMapTile {
  demands: Vec<Displacement>,
}

#[derive(Clone, Debug)]
pub struct Displacement {
  removed: Object,
  inserted: Object,
  penalty_paid: u16,
}*/

#[derive(Clone, Debug)]
pub struct RouteSearchMap {
  grid: Grid<RouteSearchMapTile>,
}

#[derive(Clone, Debug, Default)]
struct RouteSearchMapTile {
  best_route_inserting_here: MaybeRouteHead,
  best_route_insertable_from_here: MaybeRouteHead,
  obstructed: bool,
  solid_objects_here: u8,
  accumulated_solid_presence_cost: u16,
  heuristic: u8,
  this_search_objects: Vec<Object>,
  penalized_objects: Vec<(Object, usize)>,
  underground: [RouteSearchMapUnderground; 2],
  edges: [RouteSearchMapEdge; 4],
}

#[derive(Clone, Debug, Default)]
struct RouteSearchMapEdge {
  best_route_conveying_here: MaybeRouteHead,
  conveyors_locking_material: u8,
  accumulated_conveyor_locking_cost: u16,
}

#[derive(Clone, Debug, Default)]
struct RouteSearchMapUnderground {
  belts_here: u8,
  accumulated_belt_cost: u16,
}

#[derive(Clone, Debug)]
struct RouteHead {
  object: Object,
  replacing: Vec<Object>,
  route_cost: usize,
  object_bounds: Rectangle,
  route_bounds: Rectangle,
  previous: RouteOutput,
}

#[derive(Clone, Debug)]
enum MaybeRouteHead {
  Available,
  Source,
  Forbidden,
  Head(RouteHead),
}

impl Default for MaybeRouteHead {
  fn default() -> Self {
    MaybeRouteHead::Available
  }
}

impl RouteSearchMap {
  fn get_tile(&self, coordinates: Coordinates) -> Option<&RouteSearchMapTile> {
    self.grid.get(coordinates)
  }
  fn get_tile_mut(&mut self, coordinates: Coordinates) -> Option<&mut RouteSearchMapTile> {
    self.grid.get_mut(coordinates)
  }
  fn get_edge(&self, edge: DirectedEdge) -> Option<&RouteSearchMapEdge> {
    self
      .grid
      .get(edge.before_coordinates())
      .map(|tile| &tile.edges[(edge.direction() >> 1) as usize])
  }
  fn get_edge_mut(&mut self, edge: DirectedEdge) -> Option<&mut RouteSearchMapEdge> {
    self
      .grid
      .get_mut(edge.before_coordinates())
      .map(|tile| &mut tile.edges[(edge.direction() >> 1) as usize])
  }

  fn get_route_head(&self, output: &RouteOutput) -> Option<&MaybeRouteHead> {
    match output {
      RouteOutput::Conveyor(edge) => self.get_edge(*edge).map(|a| &a.best_route_conveying_here),
      RouteOutput::InsertInto(coordinates) => self
        .get_tile(*coordinates)
        .map(|a| &a.best_route_inserting_here),
      RouteOutput::InsertFrom(coordinates) => self
        .get_tile(*coordinates)
        .map(|a| &a.best_route_insertable_from_here),
    }
  }
  fn get_route_head_mut(&mut self, output: &RouteOutput) -> Option<&mut MaybeRouteHead> {
    match output {
      RouteOutput::Conveyor(edge) => self
        .get_edge_mut(*edge)
        .map(|a| &mut a.best_route_conveying_here),
      RouteOutput::InsertInto(coordinates) => self
        .get_tile_mut(*coordinates)
        .map(|a| &mut a.best_route_inserting_here),
      RouteOutput::InsertFrom(coordinates) => self
        .get_tile_mut(*coordinates)
        .map(|a| &mut a.best_route_insertable_from_here),
    }
  }

  fn extra_cost_for_object<T: ObjectTrait>(
    &self,
    //route: & RouteHistoryMap,
    object: &T,
    overlap_penalty: usize,
    replacing: &[Object],
  ) -> usize {
    let mut result = 0;
    for &coordinates in object.solid_tiles().as_ref() {
      let tile = self.get_tile(coordinates).unwrap();
      let objects_here = tile.solid_objects_here as usize
        - replacing
          .iter()
          .filter(|object| object.overlaps_solid_tile(coordinates))
          .count();
      result += objects_here as usize * overlap_penalty;
      for (penalized, penalty) in & tile.penalized_objects {
        if penalized == &object.clone().into_object() {
          result += penalty;
        }
      }
      if objects_here > 0 {
        result += tile.accumulated_solid_presence_cost as usize;
      }
    }
    for &edge in object
      .conveyor_outputs()
      .as_ref()
      .iter()
      .chain(object.conveyor_inputs().as_ref())
    {
      let edge = self.get_edge(edge).unwrap();
      result += edge.conveyors_locking_material as usize * overlap_penalty;
      if edge.accumulated_conveyor_locking_cost > 0 {
        result += edge.accumulated_conveyor_locking_cost as usize;
      }
    }
    if let Some(underground_belt) = object.as_underground_belt() {
      let index = underground_belt.horizontal() as usize;
      for coordinates in underground_belt.underground_tiles() {
        let tile = self.get_tile(coordinates).unwrap();
        result += tile.underground[index].belts_here as usize * overlap_penalty;
        if tile.underground[index].belts_here > 0 {
          result += tile.underground[index].accumulated_belt_cost as usize;
        }
      }
    }
    result
  }

  // note: intentionally does not remove the accumulated cost
  fn remove_object(&mut self, object: &Object, mid_search: bool) {
    for coordinates in object.solid_tiles() {
      let tile = self.get_tile_mut(coordinates).unwrap();
      tile.solid_objects_here -= 1;
      if mid_search {
        tile.this_search_objects.remove(
          tile
            .this_search_objects
            .iter()
            .position(|existing| existing == object)
            .unwrap(),
        );
      }
    }
    if !mid_search {
      for edge in object.conveyor_outputs() {
        let edge = self.get_edge_mut(edge).unwrap();
        edge.conveyors_locking_material -= 1;
      }
      for edge in object.conveyor_inputs() {
        let edge = self.get_edge_mut(edge).unwrap();
        edge.conveyors_locking_material -= 1;
      }
    }
    if let Object::UndergroundBelt(underground_belt) = object.clone() {
      let index = underground_belt.horizontal() as usize;
      for coordinates in underground_belt.underground_tiles() {
        let tile = self.get_tile_mut(coordinates).unwrap();
        tile.underground[index].belts_here -= 1;
      }
    }
  }
}

fn increase_accumulated_cost(cost: &mut u16, competitors: u8, penalty: usize) {
  if competitors > 0 {
    // to use this, we already paid *cost + competitors*penalty
    // let's increase exponentially from that, but not be ridiculous
    *cost += min(50, (competitors as u16 * penalty as u16 + *cost) >> 1) + 1;
    //eprintln!(" {:?} ", cost) ;
  }
}

impl<'a> RouteSearch<'a> {
  fn commit_route(&mut self, last_output: RouteOutput) {
    //eprintln!("committing; preexisting objects {}", dump_objects(&self.committed_objects));

    // First pass: Remove replaced objects (so they don't think they're being stacked)
    let mut new_committed_objects = Vec::new();
    {
      let mut iterator = last_output.clone();
      while let MaybeRouteHead::Head(head) = self
        .parameters
        .search_map
        .get_route_head(&iterator)
        .unwrap()
        .clone()
      {
        let object = &head.object;

        if head.replacing.len() > 0 {
          let mut replacing = head.replacing.clone();
          let parameters = &mut self.parameters;
          // make sure to remove only one object per thing in replacing,
          // even if two of the same object got in somehow
          self.committed_objects.retain(|existing| {
            let mut removed = false;
            replacing.retain(|r| {
              if existing == r && !removed {
                removed = true;
                false
              } else {
                true
              }
            });
            if removed {
              parameters.search_map.remove_object(existing, true);
              false
            } else {
              true
            }
          });
        }
        new_committed_objects.push(object.clone());
        //eprintln!(" {:?} ", self.committed_objects);

        iterator = head.previous.clone();
      }
    }
    
    new_committed_objects.reverse();

    for object in &new_committed_objects {
        for coordinates in object.solid_tiles() {
          let tile = self
            .parameters
            .search_map
            .get_tile_mut(coordinates)
            .unwrap();
          increase_accumulated_cost(
            &mut tile.accumulated_solid_presence_cost,
            tile.solid_objects_here,
            self.parameters.overlap_penalty,
          );
          tile.solid_objects_here += 1;
          for object in &tile.this_search_objects {
            tile.penalized_objects.push((object.clone(), 15));
          }
          tile.this_search_objects.push(object.clone());
        }
        for edge in object.conveyor_outputs() {
          let edge = self.parameters.search_map.get_edge_mut(edge).unwrap();
          increase_accumulated_cost(
            &mut edge.accumulated_conveyor_locking_cost,
            edge.conveyors_locking_material,
            self.parameters.overlap_penalty,
          );
          // do not increment tile.conveyors_locking_material yet because a route doesn't limit its OWN conveyors
        }
        for edge in object.conveyor_inputs() {
          let edge = self.parameters.search_map.get_edge_mut(edge).unwrap();
          increase_accumulated_cost(
            &mut edge.accumulated_conveyor_locking_cost,
            edge.conveyors_locking_material,
            self.parameters.overlap_penalty,
          );
          // do not increment tile.conveyors_locking_material yet because a route doesn't limit its OWN conveyors
        }
        if let Object::UndergroundBelt(underground_belt) = object.clone() {
          let index = underground_belt.horizontal() as usize;
          for coordinates in underground_belt.underground_tiles() {
            let tile = self
              .parameters
              .search_map
              .get_tile_mut(coordinates)
              .unwrap();
            increase_accumulated_cost(
              &mut tile.underground[index].accumulated_belt_cost,
              tile.underground[index].belts_here,
              self.parameters.overlap_penalty,
            );
            tile.underground[index].belts_here += 1;
          }
        }
    }
    
    self.committed_objects.extend(new_committed_objects);

    if self.finished() {
      self.finish();
    } else {
      self.clear_frontiers();
    }
  }

  fn finish(&mut self) {
    self.clear_frontiers();

    for object in &self.committed_objects {
      for coordinates in object.solid_tiles() {
        let tile = self
          .parameters
          .search_map
          .get_tile_mut(coordinates)
          .unwrap();
        tile.this_search_objects.remove(
          tile
            .this_search_objects
            .iter()
            .position(|existing| existing == object)
            .unwrap(),
        );
      }

      for edge in object.conveyor_outputs() {
        let edge = self.parameters.search_map.get_edge_mut(edge).unwrap();
        edge.conveyors_locking_material += 1;
      }
      for edge in object.conveyor_inputs() {
        let edge = self.parameters.search_map.get_edge_mut(edge).unwrap();
        edge.conveyors_locking_material += 1;
      }
    }

    let solved = self.destinations_satisfied.iter().filter(|a| **a).count();
    if solved < self.destinations_satisfied.len() {
      eprintln!(
        "Failed to find all routes! ({}/{})",
        solved,
        self.destinations_satisfied.len(),
      );
    } else {
      eprintln!(
        "Found all routes! ({}/{})",
        solved,
        self.destinations_satisfied.len(),
      );
    }
  }

  fn object_outputs<T: ObjectTrait>(&self, object: &T) -> ArrayVec<[RouteOutput; 5]> {
    let mut result = ArrayVec::new();
    for &tile in object.insertable_tiles().as_ref() {
      result.push(RouteOutput::InsertFrom(tile))
    }
    if let Some(inserter) = object.as_inserter() {
      result.push(RouteOutput::InsertInto(if self.parameters.backwards {
        inserter.input()
      } else {
        inserter.output()
      }))
    } else {
      if self.parameters.backwards {
        for input in object.conveyor_inputs().as_ref() {
          result.push(RouteOutput::Conveyor(input.reversed()))
        }
      } else {
        for &output in object.conveyor_outputs().as_ref() {
          result.push(RouteOutput::Conveyor(output))
        }
      }
    }
    result
  }

  fn route_cost(&self, route: &MaybeRouteHead) -> usize {
    match route {
      MaybeRouteHead::Head(head) => head.route_cost,
      MaybeRouteHead::Source => 0,
      _ => panic!("checking route cost of something that isn't a route"),
    }
  }

  fn route_heuristic(&self, output: &RouteOutput) -> usize {
    let heuristic_coordinates = match output {
      RouteOutput::Conveyor(edge) => edge.after_coordinates(),
      RouteOutput::InsertInto(coordinates) => *coordinates,
      RouteOutput::InsertFrom(coordinates) => *coordinates,
    };
    let heuristic = match self.parameters.search_map.get_tile(heuristic_coordinates) {
      None => 99999,
      Some(tile) => tile.heuristic as usize,
    };
    match output {
      RouteOutput::InsertInto(_) => {
        if heuristic == 0 {
          0
        } else {
          99999
        }
      }
      _ => heuristic,
    }
  }

  fn route_score(&self, output: &RouteOutput, route: &MaybeRouteHead) -> usize {
    self.route_cost(route) + self.route_heuristic(output)
  }

  fn insert_route(&mut self, output: &RouteOutput, route: MaybeRouteHead) {
    let cost = self.route_cost(&route);
    if cost >= self.parameters.max_cost {
      panic!(" Routes that are too expensive should bail out before this point ")
    }

    /*if route.output != self.parameters.destination {
      let going_into = route.output.after_coordinates();
      if self.parameters.map.obstructed (going_into) {
        return;
      }
    }*/

    let score = self.route_score(&output, &route);
    assert!(score >= self.current_frontier);
    if score >= self.parameters.max_cost {
      return;
    }
    let entry = self
      .parameters
      .search_map
      .get_route_head_mut(output)
      .unwrap();
    let better = match entry {
      MaybeRouteHead::Available => true,
      MaybeRouteHead::Head(existing) => existing.route_cost > cost,
      MaybeRouteHead::Source | MaybeRouteHead::Forbidden => false,
    };
    if better {
      *entry = route;
      self.frontiers[score].push(output.clone());
    }
  }

  fn add_object<T: ObjectTrait>(
    &mut self,
    previous: RouteOutput,
    previous_head: &MaybeRouteHead,
    object: T,
    cost: usize,
    replacing: Vec<Object>,
  ) {
    let bounds = object.physical_bounding_box();
    for &coordinates in object.solid_tiles().as_ref() {
      if self.parameters.map.obstructed(coordinates) {
        return;
      }
    }
    let new_cost = self.route_cost(previous_head)
      + cost
      + self.parameters.search_map.extra_cost_for_object(
        &object,
        self.parameters.overlap_penalty,
        &replacing,
      );
    if new_cost >= self.parameters.max_cost {
      return;
    }

    {
      let mut iterator = previous_head;
      while let MaybeRouteHead::Head(head) = iterator {
        if !head.route_bounds.overlaps(bounds) {
          break;
        }
        if bounds.overlaps(head.object_bounds) && object.physically_incompatible(&head.object) {
          return;
        }
        iterator = self
          .parameters
          .search_map
          .get_route_head(&head.previous)
          .unwrap();
      }
    }

    let outputs = self.object_outputs(&object);
    let new_head = RouteHead {
      object: object.into_object(),
      object_bounds: bounds,
      replacing: replacing,
      route_cost: new_cost,
      route_bounds: match previous_head {
        MaybeRouteHead::Head(a) => Rectangle::including_both(bounds, a.route_bounds),
        MaybeRouteHead::Source => bounds,
        _ => panic!("extending something that isn't a route"),
      },
      previous,
    };

    for output in outputs {
      self.insert_route(&output, MaybeRouteHead::Head(new_head.clone()));
    }
  }

  fn finished(&self) -> bool {
    self.destinations_satisfied.iter().all(|a| *a)
  }

  fn clear_frontiers(&mut self) {
    for frontier in &mut self.frontiers {
      for output in frontier.drain(..) {
        *self
          .parameters
          .search_map
          .get_route_head_mut(&output)
          .unwrap() = MaybeRouteHead::Available;
      }
    }
  }

  fn start_next_destination(&mut self) {
    if self.finished() {
      return;
    }

    self.current_frontier = 0;
    for (_coordinates, tile) in self.parameters.search_map.grid.tiles_mut() {
      //tile.best_route_inserting_here = MaybeRouteHead::Available;
      //tile.best_route_insertable_from_here = MaybeRouteHead::Available;
      tile.heuristic = u8::max_value();
      //for edge in &mut tile.edges {
      //  edge.best_route_conveying_here = MaybeRouteHead::Available;
      //}
    }

    let mut next_heuristic_frontier = Vec::new();
    for (destination, satisfied) in self
      .parameters
      .destinations
      .iter()
      .zip(&self.destinations_satisfied)
    {
      if *satisfied {
        continue;
      }
      match destination {
        RouteDestination::Conveyor(destination) => {
          next_heuristic_frontier.push(destination.before_coordinates());
          next_heuristic_frontier.push(destination.after_coordinates());
        }
        RouteDestination::Assembler(destination) => {
          for offset1 in -1..=1 {
            for &offset2 in &[-3, -1, 0, 1, 3] {
              next_heuristic_frontier.push([
                destination.center[0] + offset1,
                destination.center[1] + offset2,
              ]);
              next_heuristic_frontier.push([
                destination.center[0] + offset2,
                destination.center[1] + offset1,
              ]);
            }
          }
        }
      }
    }

    for which_frontier in 0..self.parameters.max_cost {
      let frontier = std::mem::replace(&mut next_heuristic_frontier, Vec::new());
      if frontier.is_empty() {
        break;
      }
      for coordinates in frontier {
        if let Some(tile) = self.parameters.search_map.get_tile_mut(coordinates) {
          if tile.heuristic > which_frontier as u8 {
            tile.heuristic = which_frontier as u8;
            for &direction in &[0, 2, 4, 6] {
              next_heuristic_frontier.push(next_coordinates(coordinates, direction));
            }
          }
        }
      }
    }
    //eprintln!(" {:?} ", self.parameters.search_map.grid.tiles().map(|t|t.1.heuristic).collect::<Vec<_>>());

    self.insert_route(
      &RouteOutput::Conveyor(self.parameters.source.clone()),
      MaybeRouteHead::Source,
    );
    for object in &self.committed_objects.clone() {
      for output in self.object_outputs(object) {
        self.insert_route(&output, MaybeRouteHead::Source);
      }

      if self.parameters.backwards {
        for input in object.conveyor_inputs() {
          self.insert_route(
            &RouteOutput::Conveyor(input.reversed()),
            MaybeRouteHead::Source,
          );
        }
      } else {
        match object {
          Object::Belt(belt) => {
            let mut input_directions = Vec::new();
            for &input in belt.conveyor_inputs().as_ref() {
              if self
                .parameters
                .search_map
                .get_tile(input.before_coordinates())
                .unwrap()
                .this_search_objects
                .iter()
                .any(|other| {
                  other
                    .conveyor_outputs()
                    .as_ref()
                    .iter()
                    .any(|&output| output == input)
                })
              {
                input_directions.push(input.direction());
              }
            }
            let mut any_next = false;
            let mut next_belts = SmallVec::<[_; 2]>::new();
            for other in &self
              .parameters
              .search_map
              .get_tile(belt.output().after_coordinates())
              .unwrap()
              .this_search_objects
            {
              for input in other.conveyor_inputs() {
                if input == belt.output() {
                  any_next = true;
                  match other {
                    Object::Belt(belt) => next_belts.push(belt.clone()),
                    _ => (),
                  }
                }
              }
            }
            if self.parameters.source.after_coordinates() == belt.position() {
              input_directions.push(self.parameters.source.direction())
            }
            if self
              .parameters
              .destinations
              .iter()
              .any(|destination| destination == &RouteDestination::Conveyor(belt.output()))
            {
              any_next = true;
            }
            if input_directions.is_empty() {
              panic!(
                "conveyors with no predecessor shouldn't exist: {:?}, {} ",
                object,
                dump_objects(&self.committed_objects)
              );
            }
            if !any_next {
              for direction in &[0, 2, 4, 6] {
                if input_directions
                  .iter()
                  .any(|input| *input == (direction + 4) % 8)
                {
                  continue;
                }

                let new_belt = Belt::new(belt.position(), *direction);
                //no cost because we are replacing something of equal cost
                let source = RouteOutput::Conveyor(DirectedEdge::from_after(
                  belt.position(),
                  input_directions[0],
                ));
                self.add_object(
                  source.clone(),
                  &MaybeRouteHead::Source,
                  new_belt,
                  0,
                  vec![object.clone()],
                );
              }
            }

            if input_directions.len() == 1 {
              let mut splitters = Vec::new();
              if input_directions[0] == belt.direction() || !any_next {
                let splitter = Splitter::from_right(belt.position(), input_directions[0]);
                let source = splitter.right_input();
                splitters.push((splitter, source, vec![object.clone()]));
                let splitter = Splitter::from_left(belt.position(), input_directions[0]);
                let source = splitter.left_input();
                splitters.push((splitter, source, vec![object.clone()]));
              } else {
                if next_belts.len() > 0
                  && input_directions[0] == next_belts[0].direction()
                  && next_belts
                    .iter()
                    .all(|next_belt| next_belt.direction() == next_belts[0].direction())
                {
                  let replace = std::iter::once(object.clone())
                    .chain(
                      next_belts
                        .iter()
                        .map(|next_belt| Object::Belt(next_belt.clone())),
                    )
                    .collect();
                  if belt.direction() == (input_directions[0] + 2) % 8 {
                    let splitter = Splitter::from_left(belt.position(), input_directions[0]);
                    let source = splitter.left_input();
                    splitters.push((splitter, source, replace));
                  } else if belt.direction() == (input_directions[0] + 6) % 8 {
                    let splitter = Splitter::from_right(belt.position(), input_directions[0]);
                    let source = splitter.right_input();
                    splitters.push((splitter, source, replace));
                  } else {
                    unreachable!()
                  }
                }
              }

              for (splitter, source, replace) in splitters {
                self.add_object(
                  RouteOutput::Conveyor(source),
                  &MaybeRouteHead::Source,
                  splitter.clone(),
                  4,
                  replace,
                );
              }
            }
          }
          _ => (),
        }
      }
    }
  }

  fn search_step(&mut self) {
    //while self.frontiers[self.current_frontier].len() > 0 {
    //eprintln!(" cost: {:?} ", self.current_frontier) ;
    //let frontier = std::mem::replace(&mut self.frontiers[self.current_frontier], Vec::new());
    //for output in frontier {
    for index in 0.. {
      let output = match self.frontiers[self.current_frontier].get(index) {
        Some(output) => output.clone(),
        None => break,
      };
      //eprintln!(" {:?} ", output) ;
      let mut satisfied_any = false;
      for index in 0..self.parameters.destinations.len() {
        if !self.destinations_satisfied[index]
          && output.satisfies(&self.parameters.destinations[index])
        {
          self.destinations_satisfied[index] = true;
          satisfied_any = true;
        }
      }
      if satisfied_any {
        self.commit_route(output);
        self.start_next_destination();

        //eprintln!("Found route!");
        //return (Some (route), index);
        return;
      }

      let head = self
        .parameters
        .search_map
        .get_route_head(&output)
        .unwrap()
        .clone();
      if self.route_score(&output, &head) != self.current_frontier {
        continue;
      }

      match output {
        RouteOutput::Conveyor(conveyor_output) => {
          let coordinates = conveyor_output.after_coordinates();
          if self.parameters.backwards {
            let belt = Belt::new(coordinates, (conveyor_output.direction() + 4) % 8);
            self.add_object(output.clone(), &head, belt, 1, Vec::new());
          } else {
            for direction in &[0, 2, 6] {
              let direction = (direction + conveyor_output.direction()) % 8;
              let belt = Belt::new(coordinates, direction);
              self.add_object(output.clone(), &head, belt, 1, Vec::new());
            }
          }
          for distance in 2..=10 {
            let underground = if self.parameters.backwards {
              UndergroundBelt::from_output(conveyor_output.reversed(), distance as u8)
            } else {
              UndergroundBelt::from_input(conveyor_output, distance as u8)
            };

            let cost = match underground.level() {
              1 => 7,
              2 => 20,
              3 => 70,
              _ => unreachable!(),
            };
            self.add_object(output.clone(), &head, underground.clone(), cost, Vec::new());
          }
        }
        RouteOutput::InsertFrom(insertable_coordinates) => {
          for direction in &[0, 2, 4, 6] {
            for length in 1..=2 {
              let inserter = Inserter::new(
                further_coordinates(insertable_coordinates, *direction, -length),
                if self.parameters.backwards {
                  (direction + 4) % 8
                } else {
                  *direction
                },
                length as u8,
              );
              //eprintln!(" {:?} ", inserter);
              let cost = 1 + length as usize * 4; //pretty expensive because they have an actual power cost;
              self.add_object(output.clone(), &head, inserter, cost, Vec::new());
            }
          }
        }
        RouteOutput::InsertInto(_output) => (), //eprintln!(" {:?} ", output), //(),//no way to extend from inserters at this time
      }
    }
    //}

    self.current_frontier += 1;
    if self.current_frontier >= self.parameters.max_cost {
      self.failed = true;
      self.finish();
    }
  }

  fn new(parameters: RouteSearchParameters) -> RouteSearch {
    let mut search = RouteSearch {
      frontiers: (0..parameters.max_cost).map(|_| Vec::new()).collect(),
      //routes: Default::default(),
      current_frontier: Default::default(),
      failed: false,
      committed_objects: Vec::new(),
      //self_so_far: Route::new (RouteOutput::Conveyor (parameters.source.clone())),
      destinations_satisfied: (0..parameters.destinations.len()).map(|_| false).collect(),
      parameters,
    };
    search.start_next_destination();
    search
  }
}

pub fn find_route(parameters: RouteSearchParameters) -> (Vec<Object>, bool) {
  let mut search = RouteSearch::new(parameters);

  while !(search.finished() || search.failed) {
    search.search_step();
  }

  (search.committed_objects, !search.failed)
}

pub fn find_routes(
  map: &RoutingMap,
  endpoints: &[(DirectedEdge, Vec<RouteDestination>, bool)],
  iterations: usize,
) -> Vec<Vec<Object>> {
  let mut current_routes: Vec<(Vec<Object>, bool)> =
    endpoints.iter().map(|_| (Vec::new(), false)).collect();
  let iterations = endpoints.len() * iterations;
  let max_cost = 6000;
  let mut search_map = RouteSearchMap {
    grid: Grid::new(map.grid.bounds().outset(1)),
  };
  for (coordinates, tile) in search_map.grid.tiles_mut() {
    for direction in 0..4 {
      let output = DirectedEdge::from_before(coordinates, direction as u8 * 2);
      if map.out_of_bounds(output.after_coordinates())
        && endpoints.iter().all(|(_, destinations, _)| {
          destinations
            .iter()
            .all(|destination| *destination != RouteDestination::Conveyor(output))
        })
      {
        tile.edges[direction].best_route_conveying_here = MaybeRouteHead::Forbidden;
      }
    }
  }
  let mut result = Vec::new();

  for iteration in 0..iterations {
    let which = iteration % endpoints.len();

    let penalty = if iteration >= iterations - endpoints.len() {
      max_cost
    } else {
      1 << min(5, iteration / endpoints.len())
    };

    //let other_routes: Vec<&Route> = current_routes.iter().enumerate().filter_map (
    //  | (index, route) | if index == which{ None } else { Some(& route.0) }).collect();

    eprintln!(" routing: {:?} ", (which, iteration, penalty));

    for object in &current_routes[which].0 {
      search_map.remove_object(object, false);
    }

    let new_route = find_route(RouteSearchParameters {
      map,
      //previous_version: Some(&current_routes [which].0),
      //other_routes: & other_routes,
      overlap_penalty: penalty,
      max_cost,
      //max_route_objects: 64,
      //conflict_history: & conflict_history,
      search_map: &mut search_map,
      source: endpoints[which].0,
      destinations: &endpoints[which].1,
      backwards: endpoints[which].2,
    });

    //conflict_history.extend_from_slice (& new_route.0.conflicts);

    current_routes[which] = new_route;
    result.push(
      current_routes
        .iter()
        .flat_map(|route| route.0.iter().cloned())
        .collect(),
    );
    /*if current_routes.iter().all(| route | route.1 && route.0.conflicts.is_empty()) {
      break;
    }*/
  }
  result
  //current_routes.into_iter().filter_map (| (route, success) | if true || success {Some (route)} else {None}).collect()
}

pub fn upgrade_from(objects: &mut [Object], source: DirectedEdge, level: u8) {
  for index in 0..objects.len() {
    if objects[index]
      .conveyor_inputs()
      .into_iter()
      .any(|input| input == source)
    {
      objects[index].upgrade_conveyor(level);
      for output in objects[index].conveyor_outputs() {
        upgrade_from(objects, output, level);
      }
    }
  }
}

pub fn upgrade_to(objects: &mut [Object], source: DirectedEdge, level: u8) {
  for index in 0..objects.len() {
    if objects[index]
      .conveyor_outputs()
      .into_iter()
      .any(|output| output == source)
    {
      objects[index].upgrade_conveyor(level);
      for input in objects[index].conveyor_inputs() {
        upgrade_to(objects, input, level);
      }
    }
  }
}

pub fn gigabase_map() -> RoutingMap {
  let mut map = RoutingMap::new(Rectangle::new([[-16, 15], [-16, 15]]));

  for x in -16i32..=15 {
    for y in -16i32..=15 {
      if (x + 36) % 16 < 8 && (y + 36) % 16 < 8 {
        *map.grid.get_mut([x, y]).unwrap() = true;
      }
    }
  } //eprintln!(" {:?} ", map);

  map
}
pub fn gigabase_electric_poles() -> Vec<Entity> {
  (&[
    [-0.5, -0.5],
    [-16.5, -16.5],
    [-16.5, 15.5],
    [15.5, 15.5],
    [15.5, -16.5],
  ])
    .iter()
    .map(|coordinates| Entity {
      name: "big-electric-pole".to_string(),
      position: Position {
        x: coordinates[0],
        y: coordinates[1],
      },
      ..Default::default()
    })
    .collect()
}

pub fn gigassembly_chunk(
  assemblers: &[(Vec<Assembler>, Entity)],
  other_objects: &[Object],
  endpoints: &[(i32, bool, usize)],
  iterations: usize,
) -> Vec<Entity> {
  let mut map = gigabase_map();
  let mut result = Vec::new();
  for (list, prototype) in assemblers {
    for assembler in list {
      for tile in assembler.shape().tiles() {
        *map.grid.get_mut(tile).unwrap() = true;
      }
      result.push(Entity {
        position: Position {
          x: assembler.center[0] as f64,
          y: assembler.center[1] as f64,
        },
        ..prototype.clone()
      });
    }
  }
  for object in other_objects {
    for tile in object.solid_tiles() {
      *map.grid.get_mut(tile).unwrap() = true;
    }
    result.extend(object.render());
  }

  let converted_endpoints: Vec<_> = endpoints
    .iter()
    .map(|&(vertical, is_output, which_assemblers)| {
      (
        DirectedEdge::from_before([16, vertical], 6),
        std::iter::once(RouteDestination::Conveyor(DirectedEdge::from_before(
          [-16, vertical],
          6,
        )))
        .chain(
          assemblers[which_assemblers]
            .0
            .iter()
            .map(|assembler| RouteDestination::Assembler(assembler.clone())),
        )
        .collect(),
        is_output,
      )
    })
    .collect();

  let mut routes = find_routes(&map, &converted_endpoints, iterations)
    .pop()
    .unwrap();

  for &(vertical, is_output, _) in endpoints {
    if is_output {
      upgrade_from(&mut routes, DirectedEdge::from_after([-16, vertical], 2), 3);
    } else {
      upgrade_to(
        &mut routes,
        DirectedEdge::from_before([-16, vertical], 6),
        3,
      );
    }
  }

  for object in routes {
    result.extend(object.render());
  }
  result.extend(gigabase_electric_poles());
  result
}

pub fn lots_of_belts() -> Vec<Entity> {
  let map = gigabase_map();

  /*let route = find_route (RouteSearchParameters {
    map: & map, previous_version: None, other_routes: & [], overlap_penalty: 5, max_cost: 600,
    source: DirectedEdge {coordinates: [16,-8], direction:6 },
    destination: DirectedEdge {coordinates: [15,8], direction:2 },
  });

  let mut result = Vec::new();
  if let Some(route) = route {
  for (_coordinates, tile) in &route.tiles {
    for entity in & tile.entities {
      result.push ((**entity).clone());
    }
  }}
  result*/

  let mut sources = Vec::new();
  for index in 0..8 {
    sources.push(DirectedEdge::from_before([16, index - 12], 6));
  }
  for index in 0..8 {
    sources.push(DirectedEdge::from_before([-17, index - 12], 2));
  }
  let mut destinations = Vec::new();
  for index in 0..8 {
    destinations.push(DirectedEdge::from_before([-16, 4 + index], 6));
  }
  //destinations.shuffle (&mut thread_rng());
  for index in 0..8 {
    destinations.push(DirectedEdge::from_before([15, 4 + index], 2));
  }
  //destinations[8..].shuffle (&mut thread_rng());
  let endpoints: Vec<_> = sources
    .into_iter()
    .zip(destinations)
    .map(|(s, d)| (s, vec![RouteDestination::Conveyor(d)], false))
    .collect();

  let routes = find_routes(&map, &endpoints, 32);

  let mut result = Vec::new();
  for route in routes {
    for object in route {
      result.extend(object.render())
    }
  }
  result
}

pub fn assemblers_thingy() -> Vec<Blueprint> {
  /*fn add_assembler(map: &mut RoutingMap, assemblers: &mut Vec<Assembler>, assembler: Assembler) {
    for tile in assembler.shape().tiles() {
      *map.grid.get_mut(tile).unwrap() = true;
    }
    assemblers.push(assembler);
  }

  let mut map = gigabase_map();
  let mut generator = ChaChaRng::from_seed([35; 32]);*/

  let mut assemblers: Vec<Assembler> = Vec::new();

  for index in 0..10 {
    let vertical = -14 + index * 3;
    assemblers.push(Assembler {
      center: [-11, vertical],
    });
    assemblers.push(Assembler {
      center: [10, vertical],
    });
  }
  for index in 0..2 {
    let vertical = 6 + index * 3;
    assemblers.push(Assembler {
      center: [-4, vertical - 1],
    });
    assemblers.push(Assembler {
      center: [3, vertical - 1],
    });
    assemblers.push(Assembler {
      center: [-4, -vertical],
    });
    assemblers.push(Assembler {
      center: [3, -vertical],
    });
  }
  //assemblers.push ( Assembler {center: [-6, -3]});
  assemblers.push(Assembler { center: [-6, 2] });
  //assemblers.push ( Assembler {center: [5, -3]});
  assemblers.push(Assembler { center: [5, 2] });
  //assemblers.push ( Assembler {center: [-6, -15]});
  //assemblers.push ( Assembler {center: [-6, 14]});
  //assemblers.push ( Assembler {center: [5, -15]});
  //assemblers.push ( Assembler {center: [5, 14]});

  /*
  while assemblers.len() < 9 {
    let assembler = Assembler {center: [generator.gen_range (-16, 16), generator.gen_range (-16, 16)]};
    if assembler.shape().tiles().all(|tile|!map.obstructed (tile)) /*&& assemblers.iter().all (| other |!Object::Assembler (other.clone()).physically_incompatible (&Object::Assembler (assembler.clone())))*/
  && !(assembler.center[0].abs() > 13) {

  assemblers.push ( assembler) ;
  }
  }
   */

  let mut endpoints = Vec::new();
  for index in 0..1 {
    endpoints.push((index - 12, false, 0));
  }
  if true {
    endpoints.push((8, true, 0));
  }

  let entities = gigassembly_chunk(
    &[(
      assemblers,
      Entity {
        name: "assembling-machine-2".to_string(),
        recipe: Some("iron-gear-wheel".to_string()),
        ..Default::default()
      },
    )],
    &[],
    &endpoints,
    8,
  );

  /*let mut result = Vec::new();
  for route in routes {
    for object in route {
      result.extend (object.render())
    }
  }
  for assembler in assemblers {
    result.extend (Object::Assembler (assembler).render())
  }*/

  /*routes
  .iter()
  .enumerate()
  .rev()
  .step_by(2)
  .map(|(index, objects)| {
    let mut entities = Vec::new();
    for object in objects {
      entities.extend(object.render());
    }
    for assembler in &assemblers {
      entities.extend(Object::Assembler(assembler.clone()).render())
    }
    Blueprint::simple(format!("Iteration {}", index), entities)
  })
  .collect()*/
  vec![Blueprint::simple(format!("Gigassembly chunk"), entities)]
}

pub fn advanced_circuits_chunk() -> Vec<Blueprint> {
  let mut cable_assemblers: Vec<Assembler> = Vec::new();
  let mut circuit_assemblers: Vec<Assembler> = Vec::new();
  let mut other_objects = Vec::new();
/*
  for &quadrant in &[
    [1, 1],
    [-1, 1],
    [-1, -1], //, [1, -1]
  ] {
    for &coordinates in &[
      [8, 14],
      [5, 14],
      [14, 10],
      [14, 7],
      [1, 5],
      [1, 8],
      [5, 1],
      [8, 1],
    ] {
      let mut coordinates = coordinates;
      coordinates[0] = coordinates[0] * quadrant[0] + min(0, quadrant[0]);
      coordinates[1] = coordinates[1] * quadrant[1] + min(0, quadrant[1]);
      circuit_assemblers.push(Assembler {
        center: coordinates,
      });
    }
    for &coordinates in &[[6, 6], [9, 9]] {
      let mut coordinates = coordinates;
      coordinates[0] = coordinates[0] * quadrant[0] + min(0, quadrant[0]);
      coordinates[1] = coordinates[1] * quadrant[1] + min(0, quadrant[1]);
      cable_assemblers.push(Assembler {
        center: coordinates,
      });
    }
    for &coordinates in &[
      [6, 11, 4],
      [7, 11, 4],
      [11, 8, 2],
      [11, 9, 2],
      [4, 7, 6],
      [4, 6, 6],
      [7, 4, 0],
      [6, 4, 0],
    ] {
      let mut coordinates = coordinates;
      coordinates[0] = coordinates[0] * quadrant[0] + min(0, quadrant[0]);
      coordinates[1] = coordinates[1] * quadrant[1] + min(0, quadrant[1]);
      if quadrant[0] > 0 && coordinates[2] % 4 == 2 {
        coordinates[2] = (coordinates[2] + 4) % 8;
      }
      if quadrant[1] > 0 && coordinates[2] % 4 == 0 {
        coordinates[2] = (coordinates[2] + 4) % 8;
      }
      other_objects.push(Object::Inserter(Inserter::new(
        [coordinates[0], coordinates[1]],
        coordinates[2] as u8,
        2,
      )));
    }
  }*/
  
  for &quadrant in &[
    [1, 1],
    [-1, 1],
    [-1, -1],
    [1, -1]
  ] {
    for &coordinates in &[
      [10, 14],
      [7, 14],
      [14, 10],
      [14, 7],
      [2, 7],
      [2, 10],
      //[5, 1],
      //[8, 1],
    ] {
      let mut coordinates = coordinates;
      coordinates[0] = coordinates[0] * quadrant[0] + min(0, quadrant[0]);
      coordinates[1] = coordinates[1] * quadrant[1] + min(0, quadrant[1]);
      circuit_assemblers.push(Assembler {
        center: coordinates,
      });
    }
    for &coordinates in &[[8, 8]] {
      let mut coordinates = coordinates;
      coordinates[0] = coordinates[0] * quadrant[0] + min(0, quadrant[0]);
      coordinates[1] = coordinates[1] * quadrant[1] + min(0, quadrant[1]);
      cable_assemblers.push(Assembler {
        center: coordinates,
      });
    }
    for &coordinates in &[
      [8, 11, 4],
      [9, 11, 4],
      [11, 8, 2],
      [11, 9, 2],
      [5, 8, 6],
      [5, 9, 6],
      //[7, 4, 0],
      //[6, 4, 0],
    ] {
      let mut coordinates = coordinates;
      coordinates[0] = coordinates[0] * quadrant[0] + min(0, quadrant[0]);
      coordinates[1] = coordinates[1] * quadrant[1] + min(0, quadrant[1]);
      if quadrant[0] > 0 && coordinates[2] % 4 == 2 {
        coordinates[2] = (coordinates[2] + 4) % 8;
      }
      if quadrant[1] > 0 && coordinates[2] % 4 == 0 {
        coordinates[2] = (coordinates[2] + 4) % 8;
      }
      other_objects.push(Object::Inserter(Inserter::new(
        [coordinates[0], coordinates[1]],
        coordinates[2] as u8,
        2,
      )));
    }
  }

  let endpoints = vec![(-6, false, 0), (-5, false, 1), (4, true, 1)];

  let entities = gigassembly_chunk(
    &[
      (
        cable_assemblers,
        Entity {
          name: "assembling-machine-2".to_string(),
          recipe: Some("copper-cable".to_string()),
          ..Default::default()
        },
      ),
      (
        circuit_assemblers,
        Entity {
          name: "assembling-machine-2".to_string(),
          recipe: Some("advanced-circuit".to_string()),
          ..Default::default()
        },
      ),
    ],
    &other_objects,
    &endpoints,
    128,
  );
  vec![Blueprint::simple(format!("Gigassembly chunk"), entities)]
}

pub fn route_blueprint_thingy() -> Vec<Blueprint> {
  //lots_of_belts()

  advanced_circuits_chunk()
}
