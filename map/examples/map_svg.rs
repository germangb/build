use map::{
    player::Player,
    sector::{SectorWalls, Sectors},
    Map,
};
use std::{fs::File, io::BufReader, path::PathBuf};
use svg::{
    node::element::{path::Data, Circle, Path},
    Document,
};

fn print_usage() {
    eprintln!("Usage: map_svg INPUT [OUTPUT]");
}

fn main() {
    pretty_env_logger::init();

    if std::env::args().any(|arg| arg == "--help") {
        print_usage();
        return;
    }

    let mut vars = std::env::args().skip(1);
    let input = PathBuf::from(vars.next().expect("Missing MAP input file."));
    let output = vars.next();

    let mut reader = BufReader::new(File::open(&input).unwrap());
    let map = map::from_reader(&mut reader).unwrap();
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

fn create_document(map: &Map) -> Document {
    let player = &map.player;
    let sectors = &map.sectors;
    let sprites = &map.sprites;
    let (min, max) = compute_bounds(sectors);
    let Player { pos_x, pos_y, .. } = &player;
    let doc = Document::new().set("viewBox", (0, 0, max[0] - min[0], max[1] - min[1]));
    let doc = sectors
        .sectors()
        .iter()
        .enumerate()
        .fold(doc, |doc, (i, _)| {
            doc.add(sector_to_path(player, sectors, min, i))
        })
        // starting position
        .add(
            Circle::new()
                .set("cx", *pos_x - min[0])
                .set("cy", *pos_y - min[1])
                .set("r", 512)
                .set("fill", "red"),
        );
    sprites.iter().fold(doc, |doc, s| {
        doc.add(
            Circle::new()
                .set("cx", s.x - min[0])
                .set("cy", s.y - min[0])
                .set("r", 128)
                .set("fill", "blue"),
        )
    })
}

fn sector_to_path(player: &Player, sectors: &Sectors, min: [i32; 2], sector: usize) -> Path {
    let (_, walls) = sectors.get(sector).unwrap();
    // set starting point of SVG path.
    let mut walls = walls.peekable();
    let mut data = Data::new();
    if let Some((l, _)) = walls.peek() {
        data = data.move_to((l.x - min[0], l.y - min[1]));
    }
    // rest of the path, using walls as segments
    let data = walls
        .fold(data, |d, (_, r)| d.line_to((r.x - min[0], r.y - min[1])))
        .close();
    #[rustfmt::skip]
        let fill = if player.sector == (sector as i16) { "#ffaaaa" } else { "white" };
    Path::new()
        .set("fill", fill)
        .set("stroke", "black")
        .set("stroke-width", 32)
        .set("d", data)
}

#[rustfmt::skip]
fn compute_bounds(sectors: &Sectors) -> ([i32; 2], [i32; 2]) {
    sectors.walls().iter().map(|w| (w.x, w.y)).fold(
        ([i32::MAX, i32::MAX], [i32::MIN, i32::MIN]),
        |(min, max), (x, y)| {
            ([min[0].min(x), min[1].min(y)],
             [max[0].max(x), max[1].max(y)])
        },
    )
}
