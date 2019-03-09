use super::blueprint::*;
use super::simplified::*;
use ordered_float::OrderedFloat;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub type Coordinates = [i32; 2];

#[derive(Clone, Debug)]
pub struct Map {
  bounds: [[i32; 2]; 2],
  obstructions: HashSet<Coordinates>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UndergroundBeltComponent {
  horizontal: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Tile {
  object: Option<Object>,
  underground: HashSet<UndergroundBeltComponent>,
}

#[derive(Clone, Debug, Default)]
pub struct Candidate {
  tiles: HashMap<Coordinates, Tile>,
  flow: FlowMap,
  score: f64,
}

/*#[derive (Clone, Debug, Default)]
pub struct Grid<T> {
  size: [usize; 2],
  origin: Coordinates,
  data: Vec<T>,
}

impl <T: Default> Grid <T> {
  fn new()->Grid <T> {

  }
}*/

impl Map {
  pub fn out_of_bounds(&self, coordinates: [i32; 2]) -> bool {
    coordinates[0] < self.bounds[0][0]
      || coordinates[0] > self.bounds[0][1]
      || coordinates[1] < self.bounds[1][0]
      || coordinates[1] > self.bounds[1][1]
  }
  pub fn obstructed(&self, coordinates: [i32; 2]) -> bool {
    self.obstructions.contains(&coordinates) || self.out_of_bounds(coordinates)
  }
}

pub struct OptimizationParameters<'a> {
  pub map: &'a Map,
  pub sources: &'a [DirectedEdge],
  pub destinations: &'a [DirectedEdge],
}
pub struct Optimizer<'a> {
  parameters: OptimizationParameters<'a>,
  //current_candidates: Rc<Candidate>,
  best_candidates: Vec<Rc<Candidate>>,
}

#[derive(Clone, Debug, Default)]
struct FlowEntry {
  amount: f64,
  previous: Option<DirectedEdge>,
}
#[derive(Clone, Debug, Default)]
pub struct FlowMap {
  map: HashMap<DirectedEdge, FlowEntry>,
  unreleased: HashSet<DirectedEdge>,
}

impl Candidate {
  pub fn evaluate<'a>(&mut self, parameters: &OptimizationParameters<'a>) {
    self.flow = FlowMap::default();
    let mut next_frontier: Vec<DirectedEdge> = parameters.sources.to_owned();

    let mut amount = 1.0;
    while next_frontier.len() > 0 {
      //eprintln!(" {:?} ", next_frontier.len()) ;
      for source in std::mem::replace(&mut next_frontier, Vec::new()) {
        self.push_source(parameters, source, amount, None);
      }
      for destination in parameters.destinations {
        self.clear_destination(parameters, *destination);
      }
      for source in std::mem::replace(&mut self.flow.unreleased, HashSet::new()) {
        let coordinates = source.after_coordinates();
        for direction in &[0, 2, 6] {
          let direction = (direction + source.direction()) % 8;
          next_frontier.push(DirectedEdge::from_before(coordinates, direction));
        }
      }
      amount *= 0.01;
    }

    self.score = parameters
      .destinations
      .iter()
      .map(|destination| match self.flow.map.get(destination) {
        None => 0.0,
        Some(entry) => entry.amount,
      })
      .sum();
  }

  fn push_source<'a>(
    &mut self,
    parameters: &OptimizationParameters<'a>,
    source: DirectedEdge,
    amount: f64,
    previous: Option<DirectedEdge>,
  ) {
    assert!(previous != Some(source));
    if self.flow.map.contains_key(&source) {
      return;
    }
    let going_into = source.after_coordinates();
    if parameters.map.obstructed(going_into)
      && parameters.map.obstructed(source.before_coordinates())
    {
      return;
    }

    self.flow.map.insert(source, FlowEntry { amount, previous });
    self.flow.unreleased.insert(source);

    if let Some(tile) = self.tiles.get(&going_into) {
      if let Some(object) = &tile.object {
        match object {
          Object::Belt(belt) => {
            self.push_source(parameters, belt.output(), amount, Some(source));
          }
          _ => unimplemented!(),
        }
      }
    }
  }

  fn clear_destination<'a>(
    &mut self,
    parameters: &OptimizationParameters<'a>,
    destination: DirectedEdge,
  ) {
    //eprintln!(" {:?} ", destination) ;
    if let Some(entry) = self.flow.map.get(&destination) {
      self.flow.unreleased.remove(&destination);
      if let Some(previous) = &entry.previous {
        self.clear_destination(parameters, *previous);
      }
    }
  }
}

impl<'a> Optimizer<'a> {
  pub fn new(parameters: OptimizationParameters<'a>) -> Optimizer<'a> {
    let mut new_candidate = Candidate::default();
    new_candidate.evaluate(&parameters);
    Optimizer {
      parameters,
      best_candidates: vec![Rc::new(new_candidate)],
    }
  }
  pub fn step(&mut self, worse_prob: f64) {
    let best = self.best_candidates.last().unwrap();
    let mut new_candidate: Candidate = (**best).clone();
    for _ in 0..thread_rng().gen_range(1, 200) {
      let coordinates = [
        thread_rng().gen_range(
          self.parameters.map.bounds[0][0],
          self.parameters.map.bounds[0][1] + 1,
        ),
        thread_rng().gen_range(
          self.parameters.map.bounds[1][0],
          self.parameters.map.bounds[1][1] + 1,
        ),
      ];
      if self.parameters.map.obstructed(coordinates) {
        continue;
      }

      // note: "max flow direction" is actually a useless metric in the current system
      let best_direction = if thread_rng().gen_range(0, 32) != 0 {
        (0..4)
          .max_by_key(|direction| {
            OrderedFloat(
              match best
                .flow
                .map
                .get(&DirectedEdge::from_before(coordinates, *direction))
              {
                None => 0.0,
                Some(entry) => entry.amount,
              },
            )
          })
          .unwrap_or_else(|| thread_rng().gen_range(0, 4) * 2)
      } else {
        thread_rng().gen_range(0, 4) * 2
      };

      new_candidate.tiles.entry(coordinates).or_default().object =
        Some(Object::Belt(Belt::new(coordinates, best_direction)));
    }
    new_candidate.evaluate(&self.parameters);
    if new_candidate.score > best.score || (random::<f64>() < worse_prob) {
      self.best_candidates.push(Rc::new(new_candidate));
    }
  }
}

pub fn blueprint_thingy() -> Vec<Blueprint> {
  let mut map = Map {
    bounds: [[-16, 15], [-16, 15]],
    obstructions: HashSet::new(),
  };

  for x in -16i32..=15 {
    for y in -16i32..=15 {
      if (x + 36) % 16 < 8 && (y + 36) % 16 < 8 {
        map.obstructions.insert([x, y]);
      }
    }
  } //eprintln!(" {:?} ", map);

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
  destinations.shuffle(&mut thread_rng());
  for index in 0..8 {
    destinations.push(DirectedEdge::from_before([15, 4 + index], 2));
  }
  destinations[8..].shuffle(&mut thread_rng());

  let mut optimizer = Optimizer::new(OptimizationParameters {
    map: &map,
    sources: &sources,
    destinations: &destinations,
  });

  let steps = 10000;
  for step in 0..steps {
    let prob = 0.0; //0.1 * (std::cmp::max(0, steps - step*5/4) as f64 / steps as f64);
    eprintln!("Step {}, {}", step, prob);

    optimizer.step(prob);
  }
  eprintln!(
    "Top score: {}",
    optimizer.best_candidates.last().unwrap().score
  );
  optimizer
    .best_candidates
    .iter()
    .rev()
    .step_by(30)
    .map(|candidate| {
      let mut entities = Vec::new();
      for (_coordinates, tile) in &candidate.tiles {
        if let Some(object) = &tile.object {
          entities.extend(object.render());
        }
      }
      Blueprint::simple(format!("Score: {}", candidate.score), entities)
    })
    .collect()
}
