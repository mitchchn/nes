; TOLEET:
;
;   "Encrypt" a null-terminated character string in rot13.
;   Maximum string length is 255 characters, plus the null term-
;   inator.
;
; Parameters:
;
;   SRC - Source string address
;   DST - Destination string address
;

ROT13   LDY #$00        ;starting index
;
LOOP    LDA (SRC),Y     ;get from source string
        BEQ DONE        ;end of string
;
        CMP #'A'        ;if lower than UC alphabet...
        BCC COPY        ;copy unchanged
;
        CMP #'z'+1      ;if greater than LC alphabet...
        BCS COPY        ;copy unchanged
;
;if lowercase
        CMP #'a'       
        BCS LOWER
;
UPPER   ADC #13         
        CMP #'Z'                   
        BMI COPY 
        SBC #25
        JMP COPY
;
LOWER   ADC #13         
        CMP #'z'                   
        BMI COPY 
        SBC #25
;
COPY    STA (DST),Y     ;store to destination string
        INY             ;bump index
        BNE LOOP        ;next character
;
; NOTE: If Y wraps the destination string will be left in an undefined
;  state.  We set carry to indicate this to the calling function.
;
        SEC             ;report string too long error &...
        RTS             ;return to caller
;
DONE    STA (DST),Y     ;terminate destination string
        CLC             ;report conversion completed &...
        RTS             ;return to caller
;

; ascii leetspeak lookup table
*= $0000
LEET    .byte 0
*= $0041
        .text "4BCD3FGHIJKLMN0PQR57UVWXYZ4[\]^_`"
        .text "4bcd3fgh!jk1mn0pqr57uvwxyz"

.END
