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
