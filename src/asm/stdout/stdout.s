.feature string_escapes

    .ZEROPAGE
    .org $02
msg:    .word 0000

    .DATA
    .org $200

prompt:     .asciiz "Enter your name: "
greetstart: .asciiz "Hello, "
greetend:   .asciiz "!\n"


name: .res 30,0

    .CODE
    .org $400

lda #<prompt
ldy #>prompt
jsr print

; get input
lda #0
sta $B000 ;readline
ldy #0
@loop:
    lda $B001,y
    sta name,y
    beq @end
    iny
    jmp @loop
@end:

lda #<greetstart
ldy #>greetstart
jsr print

lda #<name
ldy #>name
jsr print

lda #<greetend
ldy #>greetend
jsr print

jmp end

; prints null-terminated string to stdout
; lo address of string in a
; ho address of string in y
print:
    sta msg
    sty msg+1
    ldy #0
    @loop:
        lda (msg),y
        beq @end
        sta $A000
        iny
        jmp @loop
    @end:
        rts

end:
    brk