/**
 * PLL configuration
 *
 * Input clock multiplication for 
 * synchronization of sampling clock
 */
`include "../settings/settings.v"

module pll(
	input  clock_in,
	output clock_out,
	output locked
	);

SB_PLL40_CORE #(
		.FEEDBACK_PATH("SIMPLE"),
		.DIVR(`PLL_DIVR),
		.DIVF(`PLL_DIVF),
		.DIVQ(`PLL_DIVQ),
		.FILTER_RANGE(`PLL_FILT)
	) uut (
		.LOCK(locked),
		.RESETB(1'b1),
		.BYPASS(1'b0),
		.REFERENCECLK(clock_in),
		.PLLOUTCORE(clock_out)
		);

endmodule
