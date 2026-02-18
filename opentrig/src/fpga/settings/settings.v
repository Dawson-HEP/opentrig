/**
 * PLL configuration
 *
 * Input reference clock frequency
 * configuration to achieve internal
 * PLL frequency of 120 MHz
 */

`define INPUT_10MHZ     // 10 MHz clock input
// `define INPUT_40MHZ     // 40 MHz clock input


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





// compute conditional defines
`include "conditionals.v"
