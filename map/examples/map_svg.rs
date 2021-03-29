use map::{wall::SectorWalls, Map};
use std::{fs::File, io::BufReader, path::PathBuf};
use svg::{
    node::element::{path::Data, Circle, Path},
    Document,
};

fn print_usage() {
    eprintln!("Usage: map_svg INPUT [OUTPUT]");
}

fn main() {
    if std::env::args().any(|arg| arg == "--help") {
        print_usage();
        return;
    }

    let mut vars = std::env::args().skip(1);
    let input = PathBuf::from(vars.next().expect("Missing MAP input file."));
    let output = vars.next();

    let mut reader = BufReader::new(File::open(&input).unwrap());
    let map = Map::from_reader(&mut reader).unwrap();
    let document = create_document(&map);

    match output {
        Some(output) if output == "-" => svg::write(std::io::stdout(), &document),
        Some(output) => svg::write(File::create(output).unwrap(), &document),
        None => {
            let mut output = input.clone();
            output.set_extension("svg");
            svg::write(File::create(output).unwrap(), &document)
        }
    }
    .expect("Error saving SVG document");
}

fn sector_to_path(map: &Map, min: [i32; 2], sector: usize) -> Path {
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

fn compute_bounds(map: &Map) -> ([i32; 2], [i32; 2]) {
    map.sectors()
        .walls_as_slice()
        .iter()
        .map(|w| (w.x, w.y))
        .fold(
            ([i32::MAX, i32::MAX], [i32::MIN, i32::MIN]),
            |(min, max), (x, y)| {
                (
                    [min[0].min(x), min[1].min(y)],
                    [max[0].max(x), max[1].max(y)],
                )
            },
        )
}

fn create_document(map: &Map) -> Document {
    let (min, max) = compute_bounds(map);
    let doc = Document::new().set("viewBox", (0, 0, max[0] - min[0], max[1] - min[1]));
    map.sectors()
        .as_slice()
        .iter()
        .enumerate()
        .fold(doc, |doc, (i, _)| doc.add(sector_to_path(&map, min, i)))
        // starting position
        .add(
            Circle::new()
                .set("cx", map.pos_x - min[0])
                .set("cy", map.pos_y - min[1])
                .set("r", 512)
                .set("fill", "red"),
        )
}
