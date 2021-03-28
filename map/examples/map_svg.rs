use map::{sector::Bounds, Map};
use svg::{
    node::element::{path::Data, Circle, Path},
    Document,
};

fn main() {
    let map = Map::from_slice(include_bytes!("../tests/maps/VACA1.MAP")).unwrap();
    let Bounds { min, max } = map.sectors().bounds();

    let document = map.sectors().as_slice().iter().enumerate().fold(
        Document::new().set("viewBox", (0, 0, max[0] - min[0], max[1] - min[1])),
        |document, (i, sector)| {
            let (_, walls) = map.sectors().get(i).unwrap();

            // set starting point of SVG path.
            let mut walls = walls.peekable();
            let mut data = Data::new();
            if let Some((l, _)) = walls.peek() {
                data = data.move_to((l.x - min[0], l.y - min[1]));
            }

            // define the rest of the sector
            let data = walls
                .fold(data, |data, (l, r)| {
                    data.line_to((r.x - min[0], r.y - min[1]))
                })
                .close();

            document.add(
                Path::new()
                    .set(
                        "fill",
                        if (i as i16) == map.sector {
                            "#ffaaaa"
                        } else {
                            "none"
                        },
                    )
                    .set("stroke", "black")
                    .set("stroke-width", 32)
                    .set("d", data),
            )
        },
    );

    svg::save("vaca1.svg", &document).unwrap();
}
