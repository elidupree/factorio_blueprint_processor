#[allow(unused_imports)]
use image::GenericImageView;
#[allow(unused_imports)]
use rand::prelude::*;
use serde_json::Result;
#[allow(unused_imports)]
use std::fs::{self, File};

use factorio_blueprint_processor::belt_routing;
use factorio_blueprint_processor::blueprint::*;

fn main() -> Result<()> {
  /*
  let blueprint: EncodedBlueprint = EncodedBlueprint("0eNrtmj1vo0AQhv9KtDVE+8mC6yuuuCrtKYqwjRJ0DlgYRxdF/u8Hxk5ie5y8g+GKKFUEmMe7OzP78cQvYrpYZ8sqL+q7aVn+EZOXtzsrMfn97rJ9ls/Koru9yu+LdNHeq5+XmZiIvM4eRSCK9LG9qtJ8ITaByIt59ldM1OY2EFlR53Wede9vL57vivXjNKuaD7y+uaqbd+8f6nCLCMSyXDVvlUX7VQ0ptIF4bv6oaLMJTjAaxeiPKOaVMltXT9n8XFPMDqKajs7zKpt1Dy2BtNyGWaphjkvRFCU6iFI4e0jzItzF87ST8tp1sOTaHfbTEGzPYR+gCVjM7a6kKAkWTdUxjvroCKCSaLtkx4wBpmIyo0OmopjcaoiAdhom0wLttJykMfukcUg+KrhkmrmAbLGmqBE4Q6hdVhmgpR5jGpIYUUS4fizZczL+CZOpP4+/husphiOk4XpSEofCBaU0DoUrSlkcCq83YTemEgiTA5O+q08FtDKCW6nJbKLqSHveTHrUc08hY9aKqfczlD6eochhhSsqIseAqlIjsVAlVKSoETCKMwL+7ABQ05TRvdgKmf6NYUUu2cHlMZvKXmN7scMTODng+LoVUWlMZkXEZAJzt/GsvbIBiHGfLeTJoJIDkHA3A/rzNLASh1oYip+HdksNAtU4VMJQwzy5AdOXtUwmsMm0jrnMIn3nnaTcLlcNMgFYz1wbgc2bjZlM4JBhWcerhDzVSma4Y5LCWqLa5WPL8kgwnO61AThhU2bAcQuIPuU6bs3QbsA5LoYWFRHvmNhSboPOIU3eKadALNJptmju3ZTrYp5Oy3V9tV7eV+k8u/r180fzgaesWnV7i9hrHatEev0mnmTbuG+F9XUV1iVmyPfyDglS1T1PDNBs9C20vpTQ8meFlv4WWueEFjCu+xU+hPZbbLEF7LdwscWQZWoEWabHkGVmDFlmx5BljjVZn7cb5jLHhZs4P7iJi4eXZskIcs/IEWSZGlqW6RHcjRnB3djB3Q2rktw5d+Mvk1e4uzF+BHdj4hHcDUddyRHUFe5ueMfw/a4asrkMgYV7ITOCF+pnhg1iy3GPxfBC0QheyF/shWwvEwz9W/idteKYO0//TmFwgXWRCNLDiCAzjAiy/0EE3QAiSFHYcPubp4/YzcO0yaOn7G7POfNFm39bmz2t".to_string());
  let n2 = blueprint.decode()?;
  let n3 = n2.encode()?;
  //let n4 = n3.decode()?;
  //println!("{:?}", n4);
  //println!("{:?}", n3.0);
  fs::write("output.txt", &n3.0).unwrap();
  */

  /*
  fs::write("optimized_belts.txt", &
    BlueprintObject::BlueprintBook(BlueprintBook::simple("optimized belts".to_string(), optimizer::blueprint_thingy()))
    .encode()?.0).unwrap();
  */

  fs::write(
    "routed_belts.txt",
    &BlueprintObject::BlueprintBook(BlueprintBook::simple(
      "routed belts".to_string(),
      belt_routing::route_blueprint_thingy(),
    ))
    .encode()?
    .0,
  )
  .unwrap();

  /*
  // Leaving out buffer chest because the viewer I'm using doesn't display it
  let palette_names = vec![
    "wooden-chest", "iron-chest", "steel-chest",
    //"transport-belt", "fast-transport-belt", "express-transport-belt",
    "underground-belt", "fast-underground-belt", "express-underground-belt",
    "logistic-chest-active-provider", "logistic-chest-passive-provider", "logistic-chest-storage", "logistic-chest-requester", "logistic-chest-buffer",
  ];

  let mut palette_colors_source = Vec::new();
  let mut palette_colors = Vec::new() ;

  for name in & palette_names {
    let palette_image = image::open (&format!("images/{}.png", name)).unwrap();
    //let (width, height) = palette_image.dimensions();
    let mut totals = [0f64; 4];
    for (_, _, image::Rgba{data}) in palette_image.pixels() {
      for index in 0..3 {
        totals [index] += data [index] as f64*data [3] as f64/255.0;
      }
      totals [3] += data [3] as f64;
    }
    println!("Palette totals: {:?}", totals );
    palette_colors_source.push([
      totals [0] / totals[3],
      totals [1] / totals[3],
      totals [2] / totals[3],
    ]);
    /*
    let color =  exoquant::Color::new(
      (totals [0]*255.0 / totals[3]).round() as u8,
      (totals [1]*255.0 / totals[3]).round() as u8,
      (totals [2]*255.0 / totals[3]).round() as u8,
      255//(totals [3] / (width*height) as f64).round() as u8,
    );
    palette_colors.push (color);
    println!("Palette color: {},{},{},{}", color.r,color.g,color.b,color.a );*/
  }
  let mut ranges = [[1.0,0.0];3];
  for color in &palette_colors_source {
  for index in 0..3 {
  if color [index] < ranges [index] [0] { ranges [index] [0] = color [index]; }
  if color [index] > ranges [index] [1] { ranges [index] [1] = color [index]; }
  }
  }
  for color in &mut palette_colors_source {
  for index in 0..3 {
  color [index] = (color[index] - ranges [index] [0]) * 255.0 / (ranges [index] [1] - ranges [index] [0]);
  }
  }
  for color in &palette_colors_source {
  let color =  exoquant::Color::new(
  color[0].round() as u8,
  color[1].round() as u8,
  color[2].round() as u8,
  255,
  );
  palette_colors.push (color);
  println!("Palette color: {},{},{},{}", color.r,color.g,color.b,color.a );
  }

  let ditherer = exoquant::ditherer::FloydSteinberg::new(); //Ordered;
  let colorspace = exoquant::SimpleColorSpace::default();
  let remapper = exoquant::Remapper::new(& palette_colors, & colorspace, &ditherer);

  let scream_image = image::open ("/n/pfft/small_mona_lisa.png").unwrap();
  let mut scream_pixels = Vec::new();
  let (width, height) = scream_image.dimensions();
  for (_,_, image::Rgba{data}) in scream_image.pixels() {
  scream_pixels.push (exoquant::Color::new (data [0], data [1], data [2], data [3]));
  }
  let quantized = remapper.remap_usize (& scream_pixels, width as usize);
  let mut scream_entities = Vec::new();
  for ((x, y, _ /*image::Rgba{data}*/), palette_index) in scream_image.pixels().zip(quantized) {
  scream_entities.push (Entity {
  position: Position { x: (x as i32-width as i32/2) as f64, y: (y as i32-height as i32/2) as f64 },
  name: /*{
  palette_colors.iter().zip (palette_names.iter()).filter_map(|(color, name)| {
  if color == exoquant::Color::new(data [0], data [1], data [2], data [3]) {
  Some(name)
  } else {None}
  }).next().unwrap()
  }*/palette_names[palette_index].to_string(),
  underground_type: Some(UndergroundBeltOrLoaderType::Output),
  ..Default::default()
  });
  }
  let scream_object = BlueprintObject::Blueprint(Blueprint::simple("The Scream".to_string(), scream_entities));
  //println!("{:?}", scream_object);
  fs::write("the_scream_2.txt", &scream_object.encode()?.0).unwrap();
   */

  /*
  let scream_image = image::open ("/n/pfft/the_scream_factorio_2.png").unwrap();
  let mut scream_entities = Vec::new();
  let (width, height) = scream_image.dimensions();
  for x in 0..width {
    for y in 0..height {
      let pixel = scream_image.get_pixel(x,y).data;
      scream_entities.push (Entity {
        position: Position { x: (x as i32-width as i32/2) as f64, y: (y as i32-height as i32/2) as f64 },
        name: {
          /*if pixel[0] > 0 && pixel[2] > 0 {
            "logistic-chest-active-provider"
          }
          else if pixel[0] > 0 && pixel[1] > 0 {
            "logistic-chest-storage"
          }
          else if pixel[0] > 0 {
            "logistic-chest-passive-provider"
          }
          else if pixel[1] > 0 {
            "logistic-chest-buffer"
          }
          else {
            "logistic-chest-requester"
          }*/
  if pixel[0] > 0 && pixel[1] > 0 {
  "underground-belt"
  }
  else if pixel[0] > 0 {
  "fast-underground-belt"
  }
  else {
  "express-underground-belt"
  }
  }.to_string(),
  underground_type: Some(UndergroundBeltOrLoaderType::Output),
  ..Default::default()
  });
  }
  }
  let scream_object = BlueprintObject::Blueprint(Blueprint::simple("The Scream".to_string(), scream_entities));
  //println!("{:?}", scream_object);
  fs::write("the_scream.txt", &scream_object.encode()?.0).unwrap();
  println!("done The Scream");
   */

  /*
  let mut gigabus: BlueprintObject = serde_json::from_reader(File::open("gigabus.json").unwrap())?;
  gigabus.visit_blueprints (| blueprint | {
    blueprint.entities.retain(| _entity | thread_rng().gen_range(0, 16) != 0);
    for entity in &mut blueprint.entities {
      let mut change_direction = false;
      if entity.name == "express-transport-belt" || entity.name == "fast-transport-belt" {
        entity.name = "transport-belt".to_string();
        entity.name = ["transport-belt", "fast-transport-belt", "express-transport-belt"].choose(&mut rand::thread_rng()).unwrap().to_string();
        change_direction = true;
      }
      if entity.name == "express-splitter" || entity.name == "fast-splitter" {
        entity.name = "splitter".to_string();
        entity.name = ["splitter", "fast-splitter", "express-splitter"].choose(&mut rand::thread_rng()).unwrap().to_string();
      }
      if entity.name == "fast-inserter" {
        entity.name = "inserter".to_string();
        change_direction = true;
      }

      if entity.name == "express-underground-belt" || entity.name == "fast-underground-belt" {
        entity.name = "underground-belt".to_string();
        entity.name = ["underground-belt", "fast-underground-belt", "express-underground-belt"].choose(&mut rand::thread_rng()).unwrap().to_string();
        change_direction = true;
      }
      if change_direction {
        entity.direction = Some(thread_rng().gen_range(0, 8));
      }
    }
  });
  fs::write("gigabus_modified.txt", &gigabus.encode()?.0).unwrap();
  */

  println!("Done!");

  Ok(())
}
