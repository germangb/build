use map::Map;
use std::io::Cursor;

fn main() {
    let map = Map::from_slice(include_bytes!("../tests/DX-MINIDOOM.MAP")).unwrap();

    println!("pos_x = {}", map.pos_x);
    println!("pos_y = {}", map.pos_y);
    println!("pos_z = {}", map.pos_z);
    println!("angle = {}", map.angle);

    // starting sector & walls
    let sector = &map.sectors()[map.sector as usize];
    let walls = map.sectors().walls(map.sector as usize);

    for (l, r) in walls {
        print!("{:?} -> {:?}", (l.x, l.y), (r.x, r.y));
        if l.next_sector > 0 {
            print!(" (*)");
        }
        println!();
    }
}
