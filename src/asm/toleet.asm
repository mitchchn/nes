; TOLEET:
;
;   Convert a null-terminated character string to 1337sp34k.
;   Maximum string length is 255 characters, plus the null term-
;   inator.
;
; Parameters:
;
;   SRC - Source string address
;   DST - Destination string address
;

TOLEET  LDY #$00        ;starting index
;
LOOP    LDA (SRC),Y     ;get from source string
        BEQ DONE        ;end of string
;
        CMP #'A'        ;if lower than UC alphabet...
        BCC SKIP        ;copy unchanged
;
        CMP #'z'+1      ;if greater than LC alphabet...
        BCS SKIP        ;copy unchanged
;
        TAX
        LDA <LEET,X      ;retrieve 1337sp34k value from table
;
SKIP    STA (DST),Y     ;store to destination string
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
