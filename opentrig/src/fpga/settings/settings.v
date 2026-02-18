/**
 * PLL configuration
 *
 * Input reference clock frequency
 * configuration to achieve internal
 * PLL frequency of 120 MHz
 */

`define INPUT_10MHZ     // 10 MHz clock input
// `define INPUT_40MHZ     // 40 MHz clock input

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


/**
 * INTERNAL TRIGGER
 *
 * Internal trigger event handler
 * with delay
 */


// Masking channels on which internal trigger is applied
// Active: 1, Inactive: 0
// Channel order:        CH  23 --- 16 15 ---- 8 7 ----- 0
//                           |       | |       | |       |
`define        INPUT_MASK 24'b1111_1111_1111_1111_1111_1111
// `define        INPUT_MASK 24'b0000_0000_0000_0000_0000_0011;


/*
* COINCIDENCE TRIGGER SETTINGS
*  
* Triggers a pulse of adjustable
* duration upon coincidence of
* two or more inputs.
*/

// Masking channels on which coincidence is applied
// Active: 1, Inactive: 0
// Channel order:        CH  23 --- 16 15 ---- 8 7 ----- 0
//                           |       | |       | |       |
`define COINCIDENCE_MASK 24'b0000_0000_0000_0000_0000_0011

// Duration of coincidence active-high trigger output
// measured in number of full PLL clock cycles
`define COINCIDENCE_OUT_N_CYCLES 120
