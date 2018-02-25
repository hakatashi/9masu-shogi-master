use std::vec::Vec;
use ::Grid::Grid;
use ::Piece::Piece;
use ::util;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Board {
    // マスの状態
    pub grids: u64,
    // 持ち駒
    pub hands: u32,
    // 手番 (先手/後手)
    pub player: bool,
}

#[derive(PartialEq, Eq, Debug)]
struct BoardHandInfo {
    first: Vec<u8>,
    second: Vec<u8>,
}

// 座標
#[derive(PartialEq, Eq, Debug)]
pub struct Coord {
    pub x: u8,
    pub y: u8,
}

#[derive(PartialEq, Eq, Debug)]
pub struct PieceMove {
    pub piece: Piece,
    pub from: Coord,
    pub to: Coord,
    pub promote: bool,
}

impl Board {
    fn get_grid(&self, x: u8, y: u8) -> Grid {
        assert!(x < 3 && y < 3);
        Grid::from_i(((self.grids >> ((y * 3 + x) * 5)) & 0b11111) as u8)
    }

    pub fn set_grid(&self, x: u8, y: u8, grid: Grid) -> Board {
        assert!(x < 3 && y < 3);
        Board {
            grids: (self.grids & !(0b11111 << ((y * 3 + x) * 5))) | ((grid.to_i() as u64 ) << ((y * 3 + x) * 5)),
            hands: self.hands,
            player: self.player,
        }
    }

    pub fn del_grid(&self, x: u8, y: u8) -> Board {
        self.set_grid(x, y, Grid {piece: Piece::Empty, player: 0, promoted: false})
    }

    fn reverse_grids(grids: u64) -> u64 {
        let mut new_grids: u64 = 0;

        for y in 0..3 {
            let new_y = 2 - y;
            for x in 0..3 {
                let new_x = 2 - x;
                let mut grid = Grid::from_i(((grids >> ((y * 3 + x) * 5)) & 0b11111) as u8);
                grid.player = if grid.player == 0 {1} else {0};
                new_grids |= (grid.to_i() as u64) << ((new_y * 3 + new_x) * 5);
            }
        }

        new_grids
    }

    fn get_hands(&self) -> BoardHandInfo {
        let mut hands = BoardHandInfo {
            first: Vec::with_capacity(7),
            second: Vec::with_capacity(7),
        };

        let mut temp_hands = self.hands;
        for hand_type in 0..7 {
            let size = match hand_type {
                0 | 1 => 3,
                2 | 3 | 4 | 5  => 4,
                6 => 5,
                _ => panic!(),
            };
            let max_pieces = match hand_type {
                0 | 1 => 2,
                2 | 3 | 4 | 5 => 3,
                6 => 7,
                _ => panic!(),
            };
            let hand_info = util::hand_data_to_hand_info((temp_hands & ((1 << size) - 1)) as u8, max_pieces);
            temp_hands >>= size;

            hands.first.push(hand_info.first);
            hands.second.push(hand_info.second);
        }

        hands
    }

    fn set_hands(&self, hand_info: BoardHandInfo) -> Board {
        let mut hands: u32 = 0;
        let mut offset = 0;

        for hand_type in 0..7 {
            let size = match hand_type {
                0 | 1 => 3,
                2 | 3 | 4 | 5  => 4,
                6 => 5,
                _ => panic!(),
            };
            let max_pieces = match hand_type {
                0 | 1 => 2,
                2 | 3 | 4 | 5 => 3,
                6 => 7,
                _ => panic!(),
            };
            let hand_data = util::hand_info_to_hand_data(util::HandInfo {
                first: hand_info.first[hand_type],
                second: hand_info.second[hand_type],
            }, max_pieces);

            assert!(hand_data < (1 << size));

            hands |= (hand_data as u32) << offset;

            offset += size;
        }

        Board {
            grids: self.grids,
            hands: hands,
            player: self.player,
        }
    }

    pub fn add_hand(&self, player: u8, piece: Piece) -> Board {
        let mut hands = self.get_hands();

        let piece_index = match piece {
            Piece::飛車 => 0,
            Piece::角行 => 1,
            Piece::金将 => 2,
            Piece::銀将 => 3,
            Piece::桂馬 => 4,
            Piece::香車 => 5,
            Piece::歩兵 => 6,
            _ => panic!(),
        };

        match player {
            0 => hands.first[piece_index] += 1,
            1 => hands.second[piece_index] += 1,
            _ => panic!(),
        };

        self.set_hands(hands)
    }

    fn reverse_hands(&self) -> Board {
        let hands = self.get_hands();
        let new_hands = BoardHandInfo {
            first: hands.second,
            second: hands.first,
        };
        self.set_hands(new_hands)
    }

    pub fn reverse(&self) -> Board {
        Board {
            grids: Board::reverse_grids(self.grids),
            hands: self.reverse_hands().hands,
            player: self.player, // player is unused currently
        }
    }

    pub fn get_possible_moves(&self) -> Vec<PieceMove> {
        let mut moves: Vec<PieceMove> = Vec::with_capacity(128);

        for y in 0..3 {
            for x in 0..3 {
                let grid = self.get_grid(x, y);

                if grid.piece == Piece::Empty || grid.player == 1 {
                    continue;
                }

                let piece_moves = grid.get_moves();

                for piece_move in piece_moves {
                    let target_x = x as i8 + piece_move.x;
                    let target_y = y as i8 + piece_move.y;

                    // 移動先が盤外
                    if !(0..3).contains(target_x) || !(0..3).contains(target_y) {
                        continue;
                    }

                    let target_grid = self.get_grid(target_x as u8, target_y as u8);

                    // 移動先に自分の駒がある
                    if target_grid.piece != Piece::Empty && target_grid.player == 0 {
                        continue;
                    }

                    // 大駒は他の駒を飛び越えられない
                    if piece_move.x % 2 == 0 && piece_move.y % 2 == 0 {
                        // 中間点
                        let intermediate_x = x as i8 + piece_move.x / 2;
                        let intermediate_y = y as i8 + piece_move.y / 2;
                        let intermediate_grid = self.get_grid(intermediate_x as u8, intermediate_y as u8);

                        if intermediate_grid.piece != Piece::Empty {
                            continue;
                        }
                    }

                    moves.push(PieceMove {
                        from: Coord {
                            x: x,
                            y: y,
                        },
                        to: Coord {
                            x: target_x as u8,
                            y: target_y as u8,
                        },
                        piece: grid.piece,
                        promote: false,
                    });

                    if target_y == 0 && grid.is_promotable() {
                        moves.push(PieceMove {
                            from: Coord {
                                x: x,
                                y: y,
                            },
                            to: Coord {
                                x: target_x as u8,
                                y: target_y as u8,
                            },
                            piece: grid.piece,
                            promote: true,
                        });
                    }
                }
            }
        }
        moves
    }

    pub fn print(&self) {
        for y in 0..3 {
            println!("────────────────");

            for x in 0..3 {
                let grid = self.get_grid(x, y);
                print!("│{}{}{}", if grid.promoted {"!"} else {" "}, grid.piece.to_char(), if grid.player == 0 {"^"} else {"v"});
            }

            println!("│");
        }

        println!("────────────────");

        let hands = self.get_hands();

        print!("☗持ち駒");
        let mut total_count = 0;

        for (i, &count) in hands.first.iter().enumerate() {
            let piece = match i {
                0 => Piece::飛車,
                1 => Piece::角行,
                2 => Piece::金将,
                3 => Piece::銀将,
                4 => Piece::桂馬,
                5 => Piece::香車,
                6 => Piece::歩兵,
                _ => panic!(),
            };

            for _ in 0..count {
                print!(" {}", piece.to_char());
            }

            total_count += count;
        }

        if total_count == 0 {
            print!(" なし");
        }

        println!("");

        print!("☖持ち駒");
        let mut total_count = 0;

        for (i, &count) in hands.second.iter().enumerate() {
            let piece = match i {
                0 => Piece::飛車,
                1 => Piece::角行,
                2 => Piece::金将,
                3 => Piece::銀将,
                4 => Piece::桂馬,
                5 => Piece::香車,
                6 => Piece::歩兵,
                _ => panic!(),
            };

            for _ in 0..count {
                print!(" {}", piece.to_char());
            }

            total_count += count;
        }

        if total_count == 0 {
            print!(" なし");
        }

        println!("");
    }
}