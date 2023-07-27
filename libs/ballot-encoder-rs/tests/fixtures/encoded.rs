#[allow(clippy::unusual_byte_groupings)]

pub const FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_NO_VOTES: [u8; 19] = [
    // prelude
    b'V',
    b'X',
    0x02, // version
    // election hash
    0xb4,
    0xe0,
    0x78,
    0x14,
    0xb4,
    0x69,
    0x11,
    0x21,
    0x1e,
    0xc7,
    // check data
    0x04, // precinct count
    0x01, // ballot style count
    0x08, // contest count
    // ballot config
    0b10_0_1_0000,
    //││ │ │ ││││
    //││ │ │ └┴┴┴── ballot type (precinct=0)
    //││ │ └── test ballot (true)
    //││ └── ballot style index (0)
    //└┴── precinct index (2)
    0b0_0000000,
    //│ │││││││
    //│ └┴┴┴┴┴┴── vote roll call #0-6 (all false)
    //└── ballot id present? (false)
    0b0_0000000,
    //│ │││││││
    //│ └┴┴┴┴┴┴── padding bits
    //└── vote roll call #7 (false)
];

pub const FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_VOTES_IN_ALL_CONTESTS: [u8; 23] = [
    // prelude
    b'V',
    b'X',
    0x02, // version
    // election hash
    0xb4,
    0xe0,
    0x78,
    0x14,
    0xb4,
    0x69,
    0x11,
    0x21,
    0x1e,
    0xc7,
    // check data
    0x04, // precinct count
    0x01, // ballot style count
    0x08, // contest count
    // ballot config
    0b10_0_1_0000,
    //││ │ │ ││││
    //││ │ │ └┴┴┴── ballot type (precinct=0)
    //││ │ └── test ballot (true)
    //││ └── ballot style index (0)
    //└┴── precinct index (2)
    0b0_1111111,
    //│ │││││││
    //│ └┴┴┴┴┴┴── vote roll call #0-6 (all true)
    //└ ballot id present? (false)
    0b1_01_100_01,
    //│ ││ │││ ││
    //│ ││ │││ └┴── "attorney" votes (john-snow=no, mark-twain=yes)
    //│ ││ └┴┴── "controller" votes (winston-churchill=yes, oprah-winfrey=no, louis-armstrong=no)
    //│ └┴── "mayor" votes (sherlock-holmes=no, thomas-edison=yes)
    //└── vote roll call #7 (true)
    0b001_0001_0,
    //│││ ││││ │
    //│││ ││││ └── "parks-and-recreation-director" vote #0 (charles-darwin=no)
    //│││ └┴┴┴── "chief-of-police" votes (natalie-portman=no, frank-sinatra=no, andy-warhol=no, alfred-hitchcock=yes)
    //└┴┴── "public-works-director" votes (benjamin-franklin=no, robert-downey-jr=no, bill-nye=yes)
    0b010_00100,
    //│││ │││││
    //│││ └┴┴┴┴── "board-of-alderman" votes #0-4 (helen-keller=no, steve-jobs=no, nikola-tesla=yes, vincent-van-gogh=no, pablo-picasso=no)
    //└┴┴── "parks-and-recreation-director" votes #1-3 (stephen-hawking=no, johan-sebastian-bach=yes, alexander-graham-bell=no)
    0b0_00_00011,
    //│ ││ │││││
    //│ ││ └┴┴┴┴── "city-council" votes #0-4 (marie-curie=no, indiana-jones=no, mona-lisa=no, jackie-chan=yes, tim-allen=yes)
    //│ └┴── "board-of-alderman" write-in count (0)
    //└── "board-of-alderman" vote #5 (wolfgang-amadeus-mozart=no)
    0b0101_0000,
    //││││ ││││
    //││││ └┴┴┴── padding bits
    //└┴┴┴── "city-council" votes #5-8 (mark-antony=no, harriet-tubman=yes, martin-luther-king=no, marilyn-monroe=yes)
];

pub const FAMOUS_NAMES_EAST_LINCOLN_STYLE_1_WRITE_INS: [u8; 61] = [
    // prelude
    b'V',
    b'X',
    0x02, // version
    // election hash
    0xb4,
    0xe0,
    0x78,
    0x14,
    0xb4,
    0x69,
    0x11,
    0x21,
    0x1e,
    0xc7,
    // check data
    0x04, // precinct count
    0x01, // ballot style count
    0x08, // contest count
    // ballot config
    0b01_0_1_0000,
    //││ │ │ ││││
    //││ │ │ └┴┴┴── ballot type (precinct=0)
    //││ │ └── test ballot (true)
    //││ └── ballot style index (0)
    //└┴── precinct index (1)
    0b0_1111111,
    //│ │││││││
    //│ └┴┴┴┴┴┴── vote roll call #0-6 (all true)
    //└ ballot id present? (false)
    0b1_01_010_10,
    //│ ││ │││ ││
    //│ ││ │││ └┴── "attorney" votes (john-snow=yes, mark-twain=no)
    //│ ││ └┴┴── "controller" votes (winston-churchill=no, oprah-winfrey=yes, louis-armstrong=no)
    //│ └┴── "mayor" votes (sherlock-holmes=no, thomas-edison=yes)
    //└── vote roll call #7 (true)
    0b001_0000_1,
    //│││ ││││ │
    //│││ ││││ └── "chief-of-police" write-in count (1)
    //│││ └┴┴┴── "chief-of-police" votes (natalie-portman=no, frank-sinatra=no, andy-warhol=no, alfred-hitchcock=no)
    //└┴┴── "public-works-director" votes (benjamin-franklin=no, robert-downey-jr=no, bill-nye=yes)
    0b000110_01,
    //││││││ ││
    //││││││ └┴── "chief-of-police" write-in #0 char #0 (M) bits #0-1
    //└┴┴┴┴┴── "chief-of-police" write-in #0 name length (6)
    0b100_00100,
    //│││ │││││
    //│││ └┴┴┴┴── "chief-of-police" write-in #0 char #1 (E) bits #0-4
    //└┴┴── "chief-of-police" write-in #0 char #0 (M) bits #2-4
    0b10001_010,
    //│││││ │││
    //│││││ └┴┴── "chief-of-police" write-in #0 char #3 (L) bits #0-2
    //└┴┴┴┴── "chief-of-police" write-in #0 char #2 (R) bits #0-4
    0b11_01000_0,
    //││ │││││ │
    //││ │││││ └── "chief-of-police" write-in #0 char #5 (N) bit #0
    //││ └┴┴┴┴── "chief-of-police" write-in #0 char #4 (I) bits #0-4
    //└┴── "chief-of-police" write-in #0 char #3 (L) bits #3-4
    0b1101_0000,
    //││││ ││││
    //││││ └┴┴┴── "parks-and-recreation-director" votes (charles-darwin=no, stephen-hawking=no, johan-sebastian-bach=no, alexander-graham-bell=no)
    //└┴┴┴── "chief-of-police" write-in #0 char #5 (N) bits #1-4
    0b1_101000_1,
    //│ ││││││ │
    //│ ││││││ └── "parks-and-recreation-director" write-in #0 char #0 (Q) bit #0
    //│ ││││││
    //│ └┴┴┴┴┴── "parks-and-recreation-director" write-in #0 length (40)
    //└── "parks-and-recreation-director" write-in count (1)
    0b0000_1011,
    //││││ ││││
    //││││ └┴┴┴── "parks-and-recreation-director" write-in #0 char #1 (W) bits #0-3
    //└┴┴┴── "parks-and-recreation-director" write-in #0 char #0 (Q) bits #1-4
    0b0_00100_10,
    //│ │││││ ││
    //│ │││││ └┴── "parks-and-recreation-director" write-in #0 char #3 (R) bits #0-1
    //│ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #2 (E) bits #0-4
    //└── "parks-and-recreation-director" write-in #0 char #1 (W) bit #4
    0b001_10011,
    //│││ │││││
    //│││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #4 (T) bits #0-4
    //└┴┴── "parks-and-recreation-director" write-in #0 char #3 (R) bits #2-4
    0b11000_101,
    //│││││ │││
    //│││││ └┴┴── "parks-and-recreation-director" write-in #0 char #6 (U) bits #0-2
    //└┴┴┴┴── "parks-and-recreation-director" write-in #0 char #5 (Y) bits #0-4
    0b00_01000_0,
    //││ │││││ │
    //││ │││││ └── "parks-and-recreation-director" write-in #0 char #8 (O) bit #0
    //││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #7 (I) bits #0-4
    //└┴── "parks-and-recreation-director" write-in #0 char #6 (U) bits #2-4
    0b1110_0111,
    //││││ ││││
    //││││ └┴┴┴── "parks-and-recreation-director" write-in #0 char #9 (P) bits #0-3
    //└┴┴┴── "parks-and-recreation-director" write-in #0 char #8 (O) bits #1-4
    0b1_00000_10,
    //│ │││││ ││
    //│ │││││ └┴── "parks-and-recreation-director" write-in #0 char #11 (S) bits #0-1
    //│ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #10 (A) bits #0-4
    //└── "parks-and-recreation-director" write-in #0 char #9 (P) bit #4
    0b010_00011,
    //│││ │││││
    //│││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #12 (D) bits #0-4
    //└┴┴── "parks-and-recreation-director" write-in #0 char #11 (S) bits #2-4
    0b00101_001,
    //│││││ │││
    //│││││ └┴┴── "parks-and-recreation-director" write-in #0 char #14 (G) bits #0-2
    //└┴┴┴┴── "parks-and-recreation-director" write-in #0 char #13 (F) bits #0-4
    0b10_00111_0,
    //││ │││││ │
    //││ │││││ └── "parks-and-recreation-director" write-in #0 char #16 (J) bit #0
    //││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #15 (H) bits #0-4
    //└┴── "parks-and-recreation-director" write-in #0 char #14 (G) bits #2-4
    0b1001_0101,
    //││││ ││││
    //││││ └┴┴┴── "parks-and-recreation-director" write-in #0 char #17 (K) bits #0-3
    //└┴┴┴── "parks-and-recreation-director" write-in #0 char #16 (J) bits #1-4
    0b0_01011_11,
    //│ │││││ ││
    //│ │││││ └┴── "parks-and-recreation-director" write-in #0 char #19 (') bits #0-1
    //│ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #18 (L) bits #0-4
    //└── "parks-and-recreation-director" write-in #0 char #17 (K) bit #4
    0b011_11100,
    //│││ │││││
    //│││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #20 (") bits #0-4
    //└┴┴── "parks-and-recreation-director" write-in #0 char #19 (') bits #2-4
    0b11001_101,
    //│││││ │││
    //│││││ └┴┴── "parks-and-recreation-director" write-in #0 char #22 (X) bits #0-2
    //└┴┴┴┴── "parks-and-recreation-director" write-in #0 char #21 (Z) bits #0-4
    0b11_00010_1,
    //││ │││││ │
    //││ │││││ └── "parks-and-recreation-director" write-in #0 char #24 (V) bit #0
    //││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #23 (C) bits #0-4
    //└┴── "parks-and-recreation-director" write-in #0 char #22 (X) bits #2-4
    0b0101_0000,
    //││││ ││││
    //││││ └┴┴┴── "parks-and-recreation-director" write-in #0 char #25 (B) bits #0-3
    //└┴┴┴── "parks-and-recreation-director" write-in #0 char #24 (V) bits #1-4
    0b1_01101_01,
    //│ │││││ ││
    //│ │││││ └┴── "parks-and-recreation-director" write-in #0 char #27 (M) bits #0-1
    //│ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #26 (N) bits #0-4
    //└── "parks-and-recreation-director" write-in #0 char #25 (B) bit #4
    0b100_11111,
    //│││ │││││
    //│││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #28 (,) bits #0-4
    //└┴┴── "parks-and-recreation-director" write-in #0 char #27 (M) bits #2-4
    0b11110_111,
    //│││││ │││
    //│││││ └┴┴── "parks-and-recreation-director" write-in #0 char #30 (-) bits #0-2
    //└┴┴┴┴── "parks-and-recreation-director" write-in #0 char #29 (.) bits #0-4
    0b01_11010_1,
    //││ │││││ │
    //││ │││││ └── "parks-and-recreation-director" write-in #0 char #32 (") bit #0
    //││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #31 ( ) bits #0-4
    //└┴── "parks-and-recreation-director" write-in #0 char #30 (-) bits #2-4
    0b1100_1101,
    //││││ ││││
    //││││ └┴┴┴── "parks-and-recreation-director" write-in #0 char #33 (') bits #0-3
    //└┴┴┴── "parks-and-recreation-director" write-in #0 char #32 (") bits #1-4
    0b1_11110_11,
    //│ │││││ ││
    //│ │││││ └┴── "parks-and-recreation-director" write-in #0 char #35 (,) bits #0-1
    //│ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #34 (.) bits #0-4
    //└── "parks-and-recreation-director" write-in #0 char #33 (') bit #4
    0b111_11101,
    //│││ │││││
    //│││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #36 (-) bits #0-4
    //└┴┴── "parks-and-recreation-director" write-in #0 char #35 (,) bits #2-4
    0b01111_011,
    //│││││ │││
    //│││││ └┴┴── "parks-and-recreation-director" write-in #0 char #38 (O) bits #0-2
    //└┴┴┴┴── "parks-and-recreation-director" write-in #0 char #37 (P) bits #0-4
    0b10_01000_0,
    //││ │││││ │
    //││ │││││ └── "board-of-alderman" vote #0 (helen-keller=no)
    //││ └┴┴┴┴── "parks-and-recreation-director" write-in #0 char #39 (I) bits #0-4
    //└┴── "parks-and-recreation-director" write-in #0 char #38 (O) bits #2-4
    0b00101_10_0,
    //│││││ ││ │
    //│││││ ││ └── "board-of-alderman" write-in #0 length (4) bit #0
    //│││││ └┴── "board-of-alderman" write-in count (2)
    //└┴┴┴┴── "board-of-alderman" votes #1-3 (steve-jobs=no, nikola-tesla=no, vincent-van-gogh=yes, pablo-picasso=no, wolfgang-amadeus-mozart=no)
    0b00100_010,
    //│││││ │││
    //│││││ └┴┴── "board-of-alderman" write-in #0 char #0 (J) bits #0-2
    //└┴┴┴┴── "board-of-alderman" write-in #0 length (4) bits #1-5
    0b01_01110_0,
    //││ │││││ │
    //││ │││││ └── "board-of-alderman" write-in #0 char #2 (H) bit #0
    //││ └┴┴┴┴── "board-of-alderman" write-in #0 char #1 (O) bits #0-4
    //└┴── "board-of-alderman" write-in #0 char #0 (J) bits #3-4
    0b0111_0110,
    //││││ ││││
    //││││ └┴┴┴── "board-of-alderman" write-in #0 char #3 (N) bits #0-3
    //└┴┴┴── "board-of-alderman" write-in #0 char #2 (H) bits #1-4
    0b1_000101_0,
    //│ ││││││ │
    //│ ││││││ └── "board-of-alderman" write-in #1 char #0 (A) bit #0
    //│ └┴┴┴┴┴── "board-of-alderman" write-in #1 length (5)
    //└── "board-of-alderman" write-in #0 char #3 (N) bit #4
    0b0000_0101,
    //││││ ││││
    //││││ └┴┴┴── "board-of-alderman" write-in #1 char #1 (L) bits #0-3
    //└┴┴┴── "board-of-alderman" write-in #1 char #0 (A) bits #1-4
    0b1_01000_00,
    //│ │││││ ││
    //│ │││││ └┴── "board-of-alderman" write-in #1 char #3 (C) bits #0-1
    //│ └┴┴┴┴── "board-of-alderman" write-in #1 char #2 (I) bits #0-4
    //└── "board-of-alderman" write-in #1 char #1 (L) bit #4
    0b010_00100,
    //│││ │││││
    //│││ └┴┴┴┴── "board-of-alderman" write-in #1 char #4 (E) bits #0-4
    //└┴┴── "board-of-alderman" write-in #1 char #3 (C) bits #2-4
    0b00001010,
    //││││││││
    //└┴┴┴┴┴┴┴── "city-council" votes #0-7 (marie-curie=no, indiana-jones=no, mona-lisa=no, jackie-chan=no, tim-allen=yes, mark-antony=no, harriet-tubman=yes, martin-luther-king=no, marilyn-monroe=no)
    0b00_000000,
    //││ ││││││
    //││ └┴┴┴┴┴── padding bits
    //└┴── "city-council" write-in count (0)
];
