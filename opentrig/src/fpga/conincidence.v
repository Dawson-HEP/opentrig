/*
* COINCIDENCE TRIGGER
*  
* Triggers a pulse of adjustable
* duration upon coincidence of
* the two or more inputs. 
*/
`include "settings/settings.v"

module trigger_coincidence (
    input wire [23:0] inputs_async,
    input wire clk,
    output reg trigger
);
    reg [`COINCIDENCE_WINDOW_N_CYCLES:0] sr [23:0];
    wire [23:0] current;
    
    reg [23:0] sync_1;
    wire [23:0] rising = inputs_async & ~sync_1;

    reg counting;
    reg [7:0] count;

    integer i;

    always @(posedge clk) begin
        sync_1 <= inputs_async;

        for (i = 0; i < 24; i= i +1) begin
            sr[i] <= {sr[i][`COINCIDENCE_WINDOW_N_CYCLES-1:0], rising[i]};

            current[i] <= |sr[i];
        end


        // Reformulation of the if-statement below. Should be equivalent,but it
        // appears to drop some cycles.

        // if (count >= `COINCIDENCE_OUT_N_CYCLES - 1) begin
        //     counting <= 0;
        //     trigger <= 0;

        //     count <= 0;
        // end else if (counting) begin
        //     count <= count + 1;
        // end else if ((current & `COINCIDENCE_MASK) == `COINCIDENCE_MASK) begin
        //     counting <= 1;
        //     trigger <= 1;
        // end


            if (counting) begin
                count <= count + 1;

                if (count >= `COINCIDENCE_OUT_N_CYCLES - 1) begin
                    counting <= 0;
                    trigger <= 0;

                    count <= 0;
                end
            end else if ((current & `COINCIDENCE_MASK) == `COINCIDENCE_MASK) begin
                counting <= 1;
                trigger <= 1;
            end
    end

endmodule
