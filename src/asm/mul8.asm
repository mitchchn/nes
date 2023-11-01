; multiply 2 8-bit numbers
mul8
    LDA  #4
    LDX  #3
    STA $AA
    DEX
add_loop:
    ADC $AA
    DEX
    BNE add_loop
