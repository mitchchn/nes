    ; https://www.lysator.liu.se/~nisse/misc/6502-mul.html
    ;
    ; factors in factor1 and factor2

MULTIPLY LDA  #0
    LDX  #$8
    LSR  factor1
add_loop:
    BCC  no_add
    CLC
    ADC  0x1A2C
no_add:
    ROR
    ROR  factor1
    DEX
    BNE  add_loop
    STA  factor2
    ; done, high result in factor2, low result in factor1	
    LDA  factor1
    RTS