use map::{sector::Bounds, wall::SectorWalls, Map};
use std::{fs::File, io::BufReader};
use svg::{
    node::element::{path::Data, Circle, Path},
    Document,
};

fn main() {
    let mut vars = std::env::args().skip(1);
    let input = vars.next().expect("Missing MAP input file.");
    let output = vars.next();

    let mut reader = BufReader::new(File::open(input).unwrap());
    let map = Map::from_reader(&mut reader).unwrap();
    let document = create_document(&map);

    if let Some(output) = output {
        svg::write(File::create(output).unwrap(), &document)
    } else {
        svg::write(std::io::stdout(), &document)
    }
    .expect("Error saving SVG document");
}

fn sector_to_path(map: &Map, sector: usize) -> Path {
    let Bounds { min, .. } = map.sectors().bounds();
    let (_, walls) = map.sectors().get(sector).unwrap();
    // set starting point of SVG path.
    let mut walls = walls.peekable();
    let mut data = Data::new();
    if let Some((l, _)) = walls.peek() {
        data = data.move_to((l.x - min[0], l.y - min[1]));
    }
    // rest of the path, using walls as segments
    let data = walls
        .fold(data, |d, (l, r)| d.line_to((r.x - min[0], r.y - min[1])))
        .close();
    #[rustfmt::skip]
    let fill = if map.sector == (sector as i16) { "#ffaaaa" } else { "white" };
    Path::new()
        .set("fill", fill)
        .set("stroke", "black")
        .set("stroke-width", 32)
        .set("d", data)
}

fn create_document(map: &Map) -> Document {
    let Bounds { min, max } = map.sectors().bounds();
    let doc = Document::new().set("viewBox", (0, 0, max[0] - min[0], max[1] - min[1]));
    map.sectors()
        .as_slice()
        .iter()
        .enumerate()
        .fold(doc, |doc, (i, _)| doc.add(sector_to_path(&map, i)))
        // starting position
        .add(
            Circle::new()
                .set("cx", map.pos_x - min[0])
                .set("cy", map.pos_y - min[1])
                .set("r", 512)
                .set("fill", "red"),
        )
}