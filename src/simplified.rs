use super::blueprint::*;
use array_ext::Array;
use arrayvec::ArrayVec;
use std::cmp::{max, min};
use std::iter::FromIterator;

pub type Coordinates = [i32; 2];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Rectangle {
  pub bounds: [[i32; 2]; 2],
}

impl Rectangle {
  pub fn new(bounds: [[i32; 2]; 2]) -> Rectangle {
    Rectangle { bounds }
  }
  pub fn singleton(coordinates: Coordinates) -> Rectangle {
    Rectangle {
      bounds: [
        [coordinates[0], coordinates[0]],
        [coordinates[1], coordinates[1]],
      ],
    }
  }
  pub fn contains(&self, coordinates: Coordinates) -> bool {
    coordinates[0] >= self.bounds[0][0]
      && coordinates[0] <= self.bounds[0][1]
      && coordinates[1] >= self.bounds[1][0]
      && coordinates[1] <= self.bounds[1][1]
  }
  pub fn contains_rectangle(&self, other: Rectangle) -> bool {
    other.bounds[0][0] >= self.bounds[0][0]
      && other.bounds[0][1] <= self.bounds[0][1]
      && other.bounds[1][0] >= self.bounds[1][0]
      && other.bounds[1][1] <= self.bounds[1][1]
  }
  pub fn overlaps(&self, other: Rectangle) -> bool {
    other.bounds[0][1] >= self.bounds[0][0]
      && other.bounds[0][0] <= self.bounds[0][1]
      && other.bounds[1][1] >= self.bounds[1][0]
      && other.bounds[1][0] <= self.bounds[1][1]
  }

  pub fn tiles(self) -> impl Iterator<Item = Coordinates> {
    (self.bounds[0][0]..=self.bounds[0][1])
      .flat_map(move |x| (self.bounds[1][0]..=self.bounds[1][1]).map(move |y| [x, y]))
  }

  pub fn nowhere() -> Rectangle {
    Rectangle {
      bounds: [
        [i32::max_value(), i32::min_value()],
        [i32::max_value(), i32::min_value()],
      ],
    }
  }

  pub fn including_both(first: Rectangle, second: Rectangle) -> Rectangle {
    let (first, second) = (first.bounds, second.bounds);
    Rectangle {
      bounds: [
        [
          min(first[0][0], second[0][0]),
          max(first[0][1], second[0][1]),
        ],
        [
          min(first[1][0], second[1][0]),
          max(first[1][1], second[1][1]),
        ],
      ],
    }
  }

  pub fn outset(&self, distance: i32) -> Rectangle {
    let bounds = self.bounds;
    Rectangle {
      bounds: [
        [bounds[0][0] - distance, bounds[0][1] + distance],
        [bounds[1][0] - distance, bounds[1][1] + distance],
      ],
    }
  }

  fn width(&self) -> i32 {
    let bounds = self.bounds;
    (bounds[0][1] + 1 - bounds[0][0])
  }
  fn height(&self) -> i32 {
    let bounds = self.bounds;
    (bounds[1][1] + 1 - bounds[1][0])
  }
  /*fn index(&self, coordinates: Coordinates) -> Option<usize> {
    if self.contains(coordinates) {
      let bounds = self.bounds;
      let width = self.width();
      Some(((coordinates[1] - bounds[1][0]) * width + (coordinates[0] - bounds[0][0])) as usize)
    } else {
      None
    }
  }*/
  fn coordinates(&self, index: usize) -> Option<Coordinates> {
    if index < (self.width() * self.height()) as usize {
      let bounds = self.bounds;
      let width = self.width() as usize;
      Some([
        bounds[0][0] + (index % width) as i32,
        bounds[1][0] + (index / width) as i32,
      ])
    } else {
      None
    }
  }
}

#[derive(Clone, Debug)]
pub struct Grid<T> {
  bounds: Rectangle,
  minima: [usize; 2],
  size: [usize; 2],
  data: Vec<T>,
}

impl<T: Default> Grid<T> {
  pub fn new(bounds: Rectangle) -> Grid<T> {
    let size = [
      (bounds.bounds[0][1] + 1 - bounds.bounds[0][0]) as usize,
      (bounds.bounds[1][1] + 1 - bounds.bounds[1][0]) as usize,
    ];
    Grid {
      bounds,
      minima: [
        bounds.bounds[0][0] as isize as usize,
        bounds.bounds[1][0] as isize as usize,
      ],
      size,
      data: (0..(size[0] * size[1]))
        .map(|_| Default::default())
        .collect(),
    }
  }
}

impl<T> Grid<T> {
  fn index(&self, coordinates: Coordinates) -> Option<usize> {
    let horizontal = (coordinates[0] as isize as usize).wrapping_sub(self.minima[0]);
    if horizontal >= self.size[0] {
      return None;
    }
    let vertical = (coordinates[1] as isize as usize).wrapping_sub(self.minima[1]);
    if vertical >= self.size[1] {
      return None;
    }
    Some(vertical * self.size[0] + horizontal)
  }

  pub fn bounds(&self) -> Rectangle {
    self.bounds
  }
  pub fn get(&self, coordinates: Coordinates) -> Option<&T> {
    self.index(coordinates).map(|index| {
      self
        .data
        .get(index)
        .expect("grid should contain entries for all in-bounds coordinates")
    })
  }
  pub fn get_mut(&mut self, coordinates: Coordinates) -> Option<&mut T> {
    //eprintln!(" {:?} ", coordinates) ;
    if let Some(index) = self.index(coordinates) {
      //eprintln!(" {:?} ", index) ;
      Some(
        self
          .data
          .get_mut(index)
          .expect("grid should contain entries for all in-bounds coordinates"),
      )
    } else {
      None
    }
  }

  pub fn tiles(&self) -> impl Iterator<Item = (Coordinates, &T)> {
    let bounds = self.bounds;
    self
      .data
      .iter()
      .enumerate()
      .map(move |(index, value)| (bounds.coordinates(index).unwrap(), value))
    //self.bounds.tiles().map (| coordinates | (coordinates, self.get(coordinates).unwrap()))
  }
  pub fn tiles_mut<'a>(&'a mut self) -> impl Iterator<Item = (Coordinates, &'a mut T)> {
    let bounds = self.bounds;
    self
      .data
      .iter_mut()
      .enumerate()
      .map(move |(index, value)| (bounds.coordinates(index).unwrap(), value))
    //self.bounds.tiles().map (move | coordinates | (coordinates, self.get_mut(coordinates).unwrap()))
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct DirectedEdge {
  coordinates: Coordinates,
  direction: u8,
}

impl DirectedEdge {
  pub fn from_before(coordinates: [i32; 2], direction: u8) -> DirectedEdge {
    DirectedEdge {
      coordinates,
      direction,
    }
  }
  pub fn from_after(coordinates: [i32; 2], direction: u8) -> DirectedEdge {
    DirectedEdge {
      coordinates: previous_coordinates(coordinates, direction),
      direction,
    }
  }
  pub fn after_coordinates(&self) -> [i32; 2] {
    next_coordinates(self.coordinates, self.direction)
  }
  pub fn before_coordinates(&self) -> [i32; 2] {
    self.coordinates
  }
  pub fn moved(&self, direction: u8, distance: i32) -> DirectedEdge {
    DirectedEdge {
      coordinates: further_coordinates(self.coordinates, direction, distance),
      direction: self.direction,
    }
  }
  pub fn reversed(&self) -> DirectedEdge {
    Self::from_before(self.after_coordinates(), (self.direction + 4) % 8)
  }
  pub fn direction(&self) -> u8 {
    self.direction
  }
}

pub fn next_coordinates(coordinates: Coordinates, direction: u8) -> [i32; 2] {
  further_coordinates(coordinates, direction, 1)
}
pub fn previous_coordinates(coordinates: Coordinates, direction: u8) -> [i32; 2] {
  further_coordinates(coordinates, direction, -1)
}
pub fn right_coordinates(coordinates: Coordinates, direction: u8) -> [i32; 2] {
  next_coordinates(coordinates, (direction + 2) % 8)
}
pub fn left_coordinates(coordinates: Coordinates, direction: u8) -> [i32; 2] {
  next_coordinates(coordinates, (direction + 6) % 8)
}
pub fn further_coordinates(coordinates: Coordinates, direction: u8, distance: i32) -> [i32; 2] {
  let mut coordinates = coordinates;
  match direction {
    0 => coordinates[1] -= distance,
    2 => coordinates[0] += distance,
    4 => coordinates[1] += distance,
    6 => coordinates[0] -= distance,
    _ => unreachable!(),
  }
  coordinates
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Belt {
  position: Coordinates,
  direction: u8,
  level: u8,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UndergroundBelt {
  start: DirectedEdge,
  length: u8,
  level: u8,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Splitter {
  left: Coordinates,
  direction: u8,
  level: u8,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inserter {
  position: Coordinates,
  direction: u8,
  length: u8,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Assembler {
  pub center: Coordinates,
}

impl Belt {
  pub fn new(position: Coordinates, direction: u8) -> Belt {
    Belt {
      position,
      direction,
      level: 1,
    }
  }
  pub fn output(&self) -> DirectedEdge {
    DirectedEdge::from_before(self.position, self.direction)
  }
  pub fn inputs(&self) -> [DirectedEdge; 3] {
    [
      DirectedEdge::from_after(self.position, self.direction),
      DirectedEdge::from_after(self.position, (self.direction + 2) % 8),
      DirectedEdge::from_after(self.position, (self.direction + 6) % 8),
    ]
  }
  pub fn direction(&self) -> u8 {
    self.direction
  }
  pub fn position(&self) -> Coordinates {
    self.position
  }
  pub fn level(&self) -> u8 {
    self.level
  }
}

impl UndergroundBelt {
  pub fn min_level(length: u8) -> u8 {
    match length {
      2..=6 => 1,
      7..=8 => 2,
      9..=10 => 3,
      _ => unreachable!(),
    }
  }
  pub fn from_input(start: DirectedEdge, length: u8) -> UndergroundBelt {
    UndergroundBelt {
      start,
      length,
      level: UndergroundBelt::min_level(length),
    }
  }
  pub fn from_output(end: DirectedEdge, length: u8) -> UndergroundBelt {
    UndergroundBelt {
      start: end.moved(end.direction(), -(length as i32)),
      length,
      level: UndergroundBelt::min_level(length),
    }
  }
  pub fn input(&self) -> DirectedEdge {
    self.start
  }
  pub fn output(&self) -> DirectedEdge {
    self.start.moved(self.start.direction, self.length as i32)
  }
  pub fn entrance(&self) -> Coordinates {
    self.input().after_coordinates()
  }
  pub fn exit(&self) -> Coordinates {
    self.output().before_coordinates()
  }
  pub fn direction(&self) -> u8 {
    self.start.direction
  }
  pub fn length(&self) -> u8 {
    self.length
  }
  pub fn level(&self) -> u8 {
    self.level
  }

  pub fn underground_tiles(&self) -> impl Iterator<Item = Coordinates> {
    Rectangle::including_both(
      Rectangle::singleton(self.entrance()),
      Rectangle::singleton(self.exit()),
    )
    .tiles()
  }
  pub fn horizontal(&self) -> bool {
    self.direction() % 4 >= 2
  }
}
impl Splitter {
  pub fn from_left(left: Coordinates, direction: u8) -> Splitter {
    Splitter {
      left,
      direction,
      level: 1,
    }
  }
  pub fn from_right(right: Coordinates, direction: u8) -> Splitter {
    Splitter {
      left: left_coordinates(right, direction),
      direction,
      level: 1,
    }
  }
  pub fn left_part(&self) -> Coordinates {
    self.left
  }
  pub fn right_part(&self) -> Coordinates {
    right_coordinates(self.left, self.direction)
  }
  pub fn left_input(&self) -> DirectedEdge {
    DirectedEdge::from_after(self.left_part(), self.direction())
  }
  pub fn right_input(&self) -> DirectedEdge {
    DirectedEdge::from_after(self.right_part(), self.direction())
  }
  pub fn left_output(&self) -> DirectedEdge {
    DirectedEdge::from_before(self.left_part(), self.direction())
  }
  pub fn right_output(&self) -> DirectedEdge {
    DirectedEdge::from_before(self.right_part(), self.direction())
  }
  pub fn direction(&self) -> u8 {
    self.direction
  }
  pub fn level(&self) -> u8 {
    self.level
  }
}
impl Inserter {
  pub fn new(position: Coordinates, direction: u8, length: u8) -> Inserter {
    Inserter {
      position,
      direction,
      length,
    }
  }
  pub fn input(&self) -> Coordinates {
    further_coordinates(self.position, self.direction, self.length as i32)
  }
  pub fn output(&self) -> Coordinates {
    further_coordinates(self.position, self.direction, -(self.length as i32))
  }
  pub fn direction(&self) -> u8 {
    self.direction
  }
  pub fn position(&self) -> Coordinates {
    self.position
  }
}
impl Assembler {
  pub fn shape(&self) -> Rectangle {
    Rectangle::singleton(self.center).outset(1)
  }
}

/*
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ArrayWrapper <T, Item> (T, PhantomData <*const Item>);

impl <T: Array <Item>, Item: Clone> IntoIterator for ArrayWrapper <T, Item> {
  type Item = Item;
  type IntoIter = std::iter::Cloned <std::slice::Iter <T>> //ughhh
}*/

macro_rules! delegate {
  ($self: expr => $object: ident => $expression: expr) => {
    match $self {
      Object::Belt($object) => $expression,
      Object::UndergroundBelt($object) => $expression,
      Object::Splitter($object) => $expression,
      Object::Inserter($object) => $expression,
      Object::Assembler($object) => $expression,
    }
  };
}

macro_rules! objects {
  ($($Object: ident $as_fn: ident {
    fn solid_rectangles (&$self1: ident)->$SolidRectangles: ty {$($solid_rectangles: tt)*}
    fn solid_tiles (&$self2: ident)->$SolidTiles: ty {$($solid_tiles: tt)*}
    fn conveyor_outputs (&$self3: ident)->$ConveyorOutputs: ty {$($conveyor_outputs: tt)*}
    fn conveyor_inputs (&$self4: ident)->$ConveyorInputs : ty {$($conveyor_inputs: tt)*}
    fn insertable_tiles (&$self5: ident)->$InsertableTiles : ty {$($insertable_tiles: tt)*}

    $($rest:tt)*
  })*) => {


pub trait ObjectTrait: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug + Sized {
  type SolidRectangles: AsRef<[Rectangle]>;
  fn solid_rectangles (&self)->Self::SolidRectangles;

  type SolidTiles:AsRef<[Coordinates]>;
  fn solid_tiles (&self)->Self::SolidTiles;

  type ConveyorOutputs:AsRef<[DirectedEdge]>;
  fn conveyor_outputs (&self)->Self::ConveyorOutputs;

  type ConveyorInputs :AsRef<[DirectedEdge]>;
  fn conveyor_inputs (&self)->Self::ConveyorInputs ;

  type InsertableTiles :AsRef<[Coordinates]>;
  fn insertable_tiles (&self)->Self::InsertableTiles ;

  fn physical_bounding_box (&self)->Rectangle;
  fn interaction_bounding_box (&self)->Rectangle;

  fn overlaps_solid_rectangle(&self, other: Rectangle) -> bool {
    self
      .solid_rectangles()
      .as_ref().iter()
      .any(|mine| mine.overlaps(other))
  }

  fn overlaps_solid_tile(&self, coordinates: Coordinates) -> bool {
    self
      .solid_rectangles()
      .as_ref().iter()
      .any(|mine| mine.contains(coordinates))
  }

  fn physically_incompatible <Other: ObjectTrait> (&self, other: & Other) -> bool;

  fn interaction_incompatible_one_sided <Other: ObjectTrait> (&self, _other: & Other) -> bool {
    false
  }

  fn interaction_incompatible <Other: ObjectTrait> (&self, other: & Other) -> bool {
    self.physically_incompatible(other)
      || self.interaction_incompatible_one_sided(other)
      || other.interaction_incompatible_one_sided(self)
  }

  fn render(&self) -> Vec<Entity>;

  $(fn $as_fn (&self)->Option <& $Object> {None})*
  fn into_object (self)->Object;
}

    $(impl ObjectTrait for $Object {
      type SolidRectangles = $SolidRectangles;
      fn solid_rectangles (&$self1)->$SolidRectangles {$($solid_rectangles)*}

      type SolidTiles = $SolidTiles;
      fn solid_tiles (&$self2)->$SolidTiles {$($solid_tiles)*}

      type ConveyorOutputs= $ConveyorOutputs;
      fn conveyor_outputs (&$self3)->$ConveyorOutputs {$($conveyor_outputs)*}

      type ConveyorInputs = $ConveyorInputs ;
      fn conveyor_inputs (&$self4)->$ConveyorInputs {$($conveyor_inputs)*}

      type InsertableTiles = $InsertableTiles ;
      fn insertable_tiles (&$self5)->$InsertableTiles {$($insertable_tiles)*}

      fn $as_fn (&self)->Option <& $Object> {Some (&self)}
      fn into_object (self)->Object {Object::$Object (self)}

      $($rest)*
    })*

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Object {
      $($Object ($Object),)*
    }

    impl ObjectTrait for Object {
      type SolidRectangles = ArrayVec<[Rectangle; 2]>;
      fn solid_rectangles (& self)->Self::SolidRectangles {
        delegate! (self => object => {ArrayVec::from_iter (object.solid_rectangles().iter().cloned())})
      }

      type SolidTiles = ArrayVec <[Coordinates; 9]>;
      fn solid_tiles (&self)->Self::SolidTiles {
        delegate! (self => object => {ArrayVec::from_iter (object.solid_tiles().iter().cloned())})
      }

      type ConveyorOutputs= ArrayVec<[DirectedEdge; 2]>;
      fn conveyor_outputs (&self)->Self::ConveyorOutputs {
        delegate! (self => object => {ArrayVec::from_iter (object.conveyor_outputs().iter().cloned())})
      }

      type ConveyorInputs = ArrayVec<[DirectedEdge; 3]>;
      fn conveyor_inputs (&self)->Self::ConveyorInputs {
        delegate! (self => object => {ArrayVec::from_iter (object.conveyor_inputs().iter().cloned())})
      }

      type InsertableTiles = ArrayVec <[Coordinates; 9]>;
      fn insertable_tiles (&self)->Self::InsertableTiles {
        delegate! (self => object => {ArrayVec::from_iter (object.insertable_tiles().iter().cloned())})
      }

      $(fn $as_fn (&self)->Option <& $Object> {match self {Object::$Object (object) => Some (object),_=> None}})*
      fn into_object (self)->Object {self}



    fn overlaps_solid_rectangle (&self, other: Rectangle)->bool {
      delegate! (self => object => {object.overlaps_solid_rectangle (other)})
    }

    fn overlaps_solid_tile (&self, coordinates: Coordinates)->bool {
      delegate! (self => object => {object.overlaps_solid_tile (coordinates)})
    }

    fn physically_incompatible <Other: ObjectTrait> (&self, other: & Other)->bool {
      delegate! (self => object => {object.physically_incompatible (other)})
    }

    fn interaction_incompatible <Other: ObjectTrait> (&self, other: & Other) -> bool {
      delegate! (self => object => {object.interaction_incompatible (other)})
    }

    fn physical_bounding_box (&self)->Rectangle {
      delegate! (self => object => {object.physical_bounding_box ()})
    }

    fn interaction_bounding_box (&self)->Rectangle {
      delegate! (self => object => {object.interaction_bounding_box ()})
    }

    fn render(&self) -> Vec<Entity> {
delegate! (self => object => {object.render ()})
}

    }
  }
}

objects! {
  Belt as_belt {
    fn solid_rectangles (&self) -> [Rectangle; 1] {
      [Rectangle::singleton (self.position)]
    }

    fn solid_tiles (&self) -> [Coordinates; 1] {
      [self.position]
    }

    fn conveyor_outputs (&self)->[DirectedEdge; 1] {
      [self.output()]
    }

    fn conveyor_inputs (&self)->[DirectedEdge; 3] {
      self.inputs()
    }

    fn insertable_tiles (&self) -> [Coordinates; 1] {
      self.solid_tiles()
    }

    fn overlaps_solid_rectangle (&self, other: Rectangle)->bool {
      other.contains (self.position)
    }

    fn overlaps_solid_tile (&self, coordinates: Coordinates)->bool {
      coordinates == self.position
    }

    fn physically_incompatible <Other: ObjectTrait> (&self, other: & Other)->bool {
      other.overlaps_solid_tile (self.position)
    }

    fn interaction_incompatible_one_sided <Other: ObjectTrait> (&self, other: & Other) -> bool {
      other.conveyor_outputs().as_ref().iter().any (| output | output.after_coordinates() == self.position)
    }

    fn physical_bounding_box (&self)->Rectangle {
      Rectangle::singleton (self.position)
    }

    fn interaction_bounding_box (&self)->Rectangle {
      Rectangle::singleton (self.position).outset (1)
    }

    fn render(&self) -> Vec<Entity> {
      vec![Entity {
        name: match self.level {
          1 => "transport-belt",
          2 => "fast-transport-belt",
          3 => "express-transport-belt",
          _ => unreachable!(),
        }
        .to_string(),
        position: Position {
          x: self.position[0] as f64,
          y: self.position[1] as f64,
        },
        direction: Some(self.direction),
        ..Default::default()
      }]
    }
  }

  UndergroundBelt as_underground_belt {
    fn solid_rectangles (&self) -> [Rectangle; 2] {
      [Rectangle::singleton (self.entrance()), Rectangle::singleton (self.exit())]
    }

    fn solid_tiles (&self) -> [Coordinates; 2] {
      [self.entrance(), self.exit()]
    }

    fn conveyor_outputs (&self)->[DirectedEdge; 1] {
      [self.output()]
    }

    fn conveyor_inputs (&self)->[DirectedEdge; 1] {
      [self.input()]
    }

    fn insertable_tiles (&self) -> [Coordinates; 2] {
      self.solid_tiles()
    }

    fn overlaps_solid_rectangle (&self, other: Rectangle)->bool {
      other.contains (self.entrance()) || other.contains (self.exit())
    }

    fn overlaps_solid_tile (&self, coordinates: Coordinates)->bool {
      coordinates == self.entrance() || coordinates == self.exit()
    }

    fn physically_incompatible <Other: ObjectTrait> (&self, other: & Other)->bool {
      if let Some(other) = other.as_underground_belt() {
        // don't allow belt braiding, because it's rarely used in practice, and forbidding it makes automatic upgrades/downgrades easier
        //if self.level == other.level {
        let dimensions;
        if self.direction() % 4 == 0 && other.direction() % 4 == 0 {
          dimensions = Some((0, 1));
        } else if self.direction() % 4 == 2 && other.direction() % 4 == 2 {
          dimensions = Some((1, 0));
        } else {
          dimensions = None;
        }
        if let Some((narrow, long)) = dimensions {
          if self.entrance()[narrow] == other.entrance()[narrow] {
            let (first, second) = (self.entrance()[long], self.exit()[long]);
            let (third, fourth) = (other.entrance()[long], other.exit()[long]);
            if max(first, second) >= min(third, fourth) && max(third, fourth) >= min(first, second)
            {
              return true;
            }
          }
        }
        //}
      }
      other.overlaps_solid_tile (self.entrance()) || other.overlaps_solid_tile (self.exit())
    }

    fn physical_bounding_box (&self)->Rectangle {
      Rectangle::including_both(
        Rectangle::singleton(self.entrance()),
        Rectangle::singleton(self.exit()),
      )
    }

    fn interaction_bounding_box (&self)->Rectangle {
      Rectangle::including_both(
        Rectangle::singleton(self.input().before_coordinates()),
        Rectangle::singleton(self.output().after_coordinates()),
      )
    }

    fn render(&self) -> Vec<Entity> {
      let (first, second) = (self.entrance(), self.exit());
        let name = match self.level {
          1 => "underground-belt",
          2 => "fast-underground-belt",
          3 => "express-underground-belt",
          _ => unreachable!(),
        };
        vec![
          Entity {
            name: name.to_string(),
            position: Position {
              x: first[0] as f64,
              y: first[1] as f64,
            },
            direction: Some(self.direction()),
            underground_type: Some(UndergroundBeltOrLoaderType::Input),
            ..Default::default()
          },
          Entity {
            name: name.to_string(),
            position: Position {
              x: second[0] as f64,
              y: second[1] as f64,
            },
            direction: Some(self.direction()),
            underground_type: Some(UndergroundBeltOrLoaderType::Output),
            ..Default::default()
          },
        ]
    }
  }

  Splitter as_splitter {
    fn solid_rectangles (&self) -> [Rectangle; 1] {
      [Rectangle::including_both (Rectangle::singleton (self.left_part()), Rectangle::singleton (self.right_part()))]
    }

    fn solid_tiles (&self) -> [Coordinates; 2] {
      [self.left_part(), self.right_part()]
    }

    fn conveyor_outputs (&self)->[DirectedEdge; 2] {
      [self.left_output(), self.right_output()]
    }

    fn conveyor_inputs (&self)->[DirectedEdge; 2] {
      [self.left_input(), self.right_input()]
    }

    fn insertable_tiles (&self) -> [Coordinates; 2] {
      self.solid_tiles()
    }

    fn overlaps_solid_rectangle (&self, other: Rectangle)->bool {
      other.overlaps (self.solid_rectangles() [0])
    }

    fn overlaps_solid_tile (&self, coordinates: Coordinates)->bool {
      self.solid_rectangles() [0].contains (coordinates)
    }

    fn physically_incompatible <Other: ObjectTrait> (&self, other: & Other)->bool {
      other.overlaps_solid_rectangle (self.solid_rectangles().as_ref() [0])
    }

    fn interaction_incompatible_one_sided <Other: ObjectTrait> (&self, other: & Other) -> bool {
      other
        .conveyor_outputs()
        .as_ref().iter()
        .any(|&output| output == self.left_input() || output == self.right_input())
    }

    fn physical_bounding_box (&self)->Rectangle {
      Rectangle::including_both(
        Rectangle::singleton(self.left_part()),
        Rectangle::singleton(self.right_part()),
      )
    }

    fn interaction_bounding_box (&self)->Rectangle {
      Rectangle::including_both(
        Rectangle::singleton(self.left_input().before_coordinates()),
        Rectangle::singleton(self.right_output().after_coordinates()),
      )
    }

    fn render(&self) -> Vec<Entity> {

        let (first, second) = (self.left_part(), self.right_part());
        vec![Entity {
          name: match self.level {
            1 => "splitter",
            2 => "fast-splitter",
            3 => "express-splitter",
            _ => unreachable!(),
          }
          .to_string(),
          position: Position {
            x: (first[0] + second[0]) as f64 * 0.5,
            y: (first[1] + second[1]) as f64 * 0.5,
          },
          direction: Some(self.direction()),
          ..Default::default()
        }]
    }
  }


  Inserter as_inserter {
    fn solid_rectangles (&self) -> [Rectangle; 1] {
      [Rectangle::singleton (self.position)]
    }

    fn solid_tiles (&self) -> [Coordinates; 1] {
      [self.position]
    }

    fn conveyor_outputs (&self)->[DirectedEdge; 0] {
      []
    }

    fn conveyor_inputs (&self)->[DirectedEdge; 0] {
      []
    }

    fn insertable_tiles (&self) -> [Coordinates; 0] {
      []
    }

    fn overlaps_solid_rectangle (&self, other: Rectangle)->bool {
      other.contains (self.position)
    }

    fn overlaps_solid_tile (&self, coordinates: Coordinates)->bool {
      coordinates == self.position
    }

    fn physically_incompatible <Other: ObjectTrait> (&self, other: & Other)->bool {
      other.overlaps_solid_tile (self.position)
    }

    fn physical_bounding_box (&self)->Rectangle {
      Rectangle::singleton (self.position)
    }

    fn interaction_bounding_box (&self)->Rectangle {
      Rectangle::including_both(
        Rectangle::singleton(self.input()),
        Rectangle::singleton(self.output()),
      )
    }

    fn render(&self) -> Vec<Entity> {
      vec![Entity {
        name: match self.length {
          1 => "inserter",
          2 => "long-handed-inserter",
          _ => unreachable!(),
        }
        .to_string(),
        position: Position {
          x: self.position[0] as f64,
          y: self.position[1] as f64,
        },
        direction: Some(self.direction),
        ..Default::default()
      }]
    }
  }



  Assembler as_assembler {
    fn solid_rectangles (&self) -> [Rectangle; 1] {
      [self.shape()]
    }

    fn solid_tiles (&self) -> [Coordinates; 9] {
      Array::from_iter (self.shape().tiles()).unwrap()
    }

    fn conveyor_outputs (&self)->[DirectedEdge; 0] {
      []
    }

    fn conveyor_inputs (&self)->[DirectedEdge; 0] {
      []
    }

    fn insertable_tiles (&self) -> [Coordinates; 9] {
      self.solid_tiles()
    }

    fn overlaps_solid_rectangle (&self, other: Rectangle)->bool {
      other.overlaps (self.shape())
    }

    fn overlaps_solid_tile (&self, coordinates: Coordinates)->bool {
      self.shape().contains (coordinates)
    }

    fn physically_incompatible <Other: ObjectTrait> (&self, other: & Other)->bool {
      other.overlaps_solid_rectangle (self.shape())
    }

    fn physical_bounding_box (&self)->Rectangle {
      self.shape()
    }

    fn interaction_bounding_box (&self)->Rectangle {
      self.shape()
    }

    fn render(&self) -> Vec<Entity> {
      vec![Entity {
        name: "assembling-machine-1".to_string(),
        position: Position {
          x: self.center[0] as f64,
          y: self.center[1] as f64,
        },
        direction: Some(0),
        ..Default::default()
      }]
    }
  }


}

impl Object {
  pub fn upgrade_conveyor(&mut self, level: u8) {
    match self {
      Object::Belt(object) => object.level = max(object.level, level),
      Object::UndergroundBelt(object) => object.level = max(object.level, level),
      Object::Splitter(object) => object.level = max(object.level, level),
      _ => (),
    }
  }
}

pub fn dump_objects(objects: &[Object]) -> String {
  let entities = objects.iter().flat_map(|object| object.render()).collect();
  BlueprintObject::Blueprint(Blueprint::simple("routed belts".to_string(), entities))
    .encode()
    .unwrap()
    .0
}
