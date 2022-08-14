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
        CMP #'a'        ;if lowercase...
        BCS LOWER       ;do lowercase rot13
;
UPPER   ADC #13         ;rotate char right by 13         
        CMP #'Z'        ;if still within UC alphabet...                
        BMI COPY        ;copy rotated char
        CLC             ;clear carry bit
        SBC #25         ;rotate char left by 25 (equivalent to Char-('Z'+'A'))
        JMP COPY        ;copy rotated char
;
LOWER   ADC #13         ;same as for upper         
        CMP #'z'                   
        BMI COPY
        CLC
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
