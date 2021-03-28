use map::Map;
use svg::{
    node::element::{path::Data, Path},
    Document,
};

fn main() {
    let map = Map::from_slice(include_bytes!("../tests/DUKEDC1.MAP")).unwrap();
    let (min, max) = compute_bounds(&map);

    let mut document = Document::new().set("viewBox", (0, 0, max[0] - min[0], max[1] - min[1]));

    for (i, sector) in map.sectors().as_slice().iter().enumerate() {
        let (_, walls) = map.sectors().get(i).unwrap();

        // define SVG path
        let data = walls
            .fold(Data::new(), |data, (l, r)| {
                data.move_to((l.x - min[0], l.y - min[1]))
                    .line_to((r.x - min[0], r.y - min[1]))
            })
            .close();

        #[rustfmt::skip]
        let stroke = if i as i16 == map.sector { "red" } else { "black" };
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", stroke)
            .set("stroke-width", 32)
            .set("d", data);

        document = document.add(path);
    }

    svg::save("map.svg", &document).unwrap();
}

fn compute_bounds(map: &Map) -> ([i32; 2], [i32; 2]) {
    map.sectors().walls_as_slice().iter().fold(
        ([i32::MAX, i32::MAX], [i32::MIN, i32::MIN]),
        |(mut min, mut max), wall| {
            if wall.x < min[0] {
                min[0] = wall.x
            }
            if wall.y < min[1] {
                min[1] = wall.y
            }
            if wall.x > max[0] {
                max[0] = wall.x
            }
            if wall.y > max[1] {
                max[1] = wall.y
            }
            (min, max)
        },
    )
}
