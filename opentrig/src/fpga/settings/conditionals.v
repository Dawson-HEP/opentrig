// PLL clock multipliers
`ifdef INPUT_10MHZ
    `define PLL_DIVR 4'b0000
    `define PLL_DIVF 7'b1011111
    `define PLL_DIVQ 3'b011
    `define PLL_FILT 3'b001
`endif
`ifdef INPUT_40MHZ
    `define PLL_DIVR 4'b0000
    `define PLL_DIVF 7'b0010111
    `define PLL_DIVQ 3'b011
    `define PLL_FILT 3'b011
`endif
