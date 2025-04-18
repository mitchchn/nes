;───────────────────────────────────────────────────────────────────────────────
;
;  S N A K E  –  32 × 32 byte‑mapped screen, pure‑RAM 6502
;  ───────────────────────────────────────────────────────────────────────────
;  • Controls   : W‑A‑S‑D  (up / left / down / right)
;  • Screen RAM : $0200–$05FF   (32 bytes/row × 32 rows)
;  • RNG        : read from $FE   (any changing value is fine)
;  • Keyboard   : last ASCII key in $FF  (0 if none)
;  • Build      : ca65 snake.s -o snake.o
;                 ld65 -C snake.cfg -o snake.bin snake.o
;
;  A very small demonstration:  three‑segment snake, single food item,
;  simple self‑collision, no walls (wrap is fatal).  Everything lives in RAM;
;  the ROM image only contains the code and vectors.
;
;                       --o>   <- our little ASCII friend
;───────────────────────────────────────────────────────────────────────────────

        .setcpu "6502"

;───────────────────────── Z E R O P A G E  ───────────────────────────────────
.segment "ZEROPAGE"

RAND        = $FE              ; random byte supplied by the host/emulator
KEYREG      = $FF              ; ASCII of the most‑recent key

SCREEN_BASE = $0200            ; top‑left of the play‑field

frameDelay:    .res 1          ; how many “slices” to burn each frame

framesPerMove: .res 1          ; how many frames to wait between moves
moveCounter:   .res 1          ; counts frames since last move


foodColor: .res 1              ; current random food color
foodColorCounter: .res 1       ; counts frames since last food color change

foodPos:    .res 2             ; 16‑bit pointer to the food byte

dirBits:    .res 1             ; 1=Up  2=Right  4=Down  8=Left
nextDirBits: .res 1 
snakeLen:   .res 1             ; length *in bytes* (2 per segment)

ptrLo:      .res 1             ; scratch pointer for screen clear
ptrHi:      .res 1


headIdx:   .res 2             ; 16-bit pointer to the snake head (head of ring buffer)
tailIdx:   .res 2             ; 16-bit pointer to the snake tail (tail of ring buffer)

snakeBuf    = $10              ; ring buffer of 16‑bit screen addresses
                               ; we reserve 64 bytes → 32 segments

;───────────────────────── C O N S T A N T S ──────────────────────────────────
HEAD_CHAR   = $01              ; non‑zero so it shows up
EMPTY_CHAR  = $00

;───────────────────────── C O D E  S T A R T S  H E R E ──────────────────────
.segment "CODE"

;────────────────────────────────  RESET ENTRY  ───────────────────────────────
Start:
        ; clear play‑field, build snake & first food, then fall into main loop
        JSR ClearScreen
        JSR InitGame
        JSR DrawInitialSnake
        JSR DrawFood

;───────────────────────────────  M A I N  L O O P  ──────────────────────────
; game loop runs once per frame (expecting 60 FPS), but the snake advances at a set speed
MainLoop:
        JSR PollKeys           ; maybe change direction
        JSR TickGame           ; maybe move snake
        JSR DrawFood           ; flashing food byte
        JSR Delay              ; crude frame limiter
        JMP MainLoop

TickGame:
        DEC moveCounter      
        BEQ MoveSnake
        RTS
MoveSnake:
        LDA nextDirBits        ; update direction based on last input
        STA dirBits
        LDA framesPerMove      ; reset move counter
        STA moveCounter
        JSR AdvanceSnake       ; shift body & move head
        JSR EatOrDie           ; grow or die
        RTS

;───────────────────────────  C L E A R  S C R E E N  ─────────────────────────
;  Writes EMPTY_CHAR to every byte in $0200–$05FF (1024 bytes)
ClearScreen:
        LDA #<SCREEN_BASE
        STA ptrLo
        LDA #>SCREEN_BASE
        STA ptrHi
        LDX #$04                  ; 4 pages × 256 bytes = 1024
ClearPage:
        LDY #$00
ClearByte:
        LDA #EMPTY_CHAR
        STA (ptrLo),Y
        INY
        BNE ClearByte             ; loop within page
        INC ptrHi
        DEX
        BNE ClearPage
        RTS

;─────────────────────────────  I N I T  G A M E  ────────────────────────────
;  * Resets and clears the ring buffer
;  * Resets direction, length and key latch
;  * Builds the initial three‑segment snake in the centre row
;  * Drops the first piece of food
InitGame:
        ; reset snake head and tail position
        LDA #<snakeBuf
        STA headIdx
        LDA #>snakeBuf
        sta headIdx+1
        ; zero the whole 64‑byte ring buffer so stale data can’t survive reset
        LDX #$3F
ZeroBuf:
        LDA #$00
        STA snakeBuf,X
        DEX
        BPL ZeroBuf

        ; forget any key pressed before RESET (prevents an instant turn)
        LDA #$00
        STA KEYREG

        ; n = CPU_MHz × 13   (rounded)
        ; 26 × 1 279 ≈ 33 300 cycles  (~16.7 ms @ 2 MHz)
        LDA #26   
        STA frameDelay

        ; start heading right
        LDA #$02
        STA dirBits
        STA nextDirBits

        ; current length = 0 (we’ll append three segments)
        LDA #$00
        STA snakeLen

        ; ── append first three segments starting $040F - $0411
        LDA #$11
        LDY #$04
        JSR AppendHead

        LDA #$10
        LDY #$04
        JSR AppendHead

        LDA #$0F
        LDY #$04
        JSR AppendHead

        ; ── set initial speed —
        ; move once every 10 frames
        LDA #$0A
        STA framesPerMove
        STA moveCounter

        LDA #$01
        STA foodColorCounter

        ; drop a food byte somewhere random
        JSR PlaceFood
        RTS


AppendHead:
        LDX snakeLen
        STA snakeBuf,X
        INX
        STY snakeBuf,X
        INC snakeLen
        INC snakeLen
        RTS

;────────────────────────────  D R A W  S N A K E  ───────────────────────────
;  Paints every segment in the buffer (used once at reset)
DrawInitialSnake:
        LDX #$00
DrawSeg:
        LDY #$00
        LDA #HEAD_CHAR
        STA (snakeBuf,X)
        INX
        INX
        CPX snakeLen
        BNE DrawSeg
        RTS

;────────────────────────────  P L A C E  F O O D  ───────────────────────────
;  Uniform random position over the 32×32 field
;  low = 0‑255 (column + low 3 bits of row),  high = $02–$05 (row page)
PlaceFood:
        LDA RAND
        STA foodPos
        LDA RAND
        AND #%11                  ; mask to keep 2 low bits, which could be 0‑3
        CLC
        ADC #$02                  ; → $02‑$05
        STA foodPos+1
        RTS
;────────────────────────────  D R A W  F O O D  ─────────────────────────────
;  Changes the food color every 255 frames
DrawFood:
        JSR CheckFoodColor
        LDY #$00
        LDA foodColor
        STA (foodPos),Y
        RTS
CheckFoodColor:
        DEC foodColorCounter
        BEQ GetNewFoodColor
        RTS
; get a color value between 1-15
GetNewFoodColor: 
        LDA RAND
        AND #%1111              ; mask to keep lower 4 bits (0-15)
        BEQ GetNewFoodColor     ; if we get 0 (black), retry
SaveNewFoodColor:
        STA foodColor
        LDA #$FF
        STA foodColorCounter
        RTS

;───────────────────────────  P O L L  K E Y S  ──────────────────────────────
;  Reads KEYREG and updates requested direction, disallowing 180° reversals
PollKeys:
        LDA KEYREG
        CMP #'w'
        BEQ KeyUp
        CMP #'d'
        BEQ KeyRight
        CMP #'s'
        BEQ KeyDown
        CMP #'a'
        BEQ KeyLeft
        RTS                        ; no recognised key

KeyUp:
        LDA dirBits
        AND #$04                   ; currently Down?
        BNE KeyDone
        LDA #$01
        STA nextDirBits
KeyDone:
        RTS

KeyRight:
        LDA dirBits
        AND #$08                   ; currently Left?
        BNE KeyDone
        LDA #$02
        STA nextDirBits
        RTS

KeyDown:
        LDA dirBits
        AND #$01                   ; currently Up?
        BNE KeyDone
        LDA #$04
        STA nextDirBits
        RTS

KeyLeft:
        LDA dirBits
        AND #$02                   ; currently Right?
        BNE KeyDone
        LDA #$08
        STA nextDirBits
        RTS

;────────────────────────────  E A T / D I E  ────────────────────────────────
EatOrDie:
        ; ── food? ──
        LDA foodPos
        CMP snakeBuf
        BNE CheckSelf
        LDA foodPos+1
        CMP snakeBuf+1
        BNE CheckSelf
        ; grow snake
        INC snakeLen               ; grow by one segment (2 bytes)
        INC snakeLen
        JSR GetNewFoodColor
        JSR PlaceFood

CheckSelf:
        ; ── self‑collision? ──
        LDX #$02                   ; first body segment
SelfLoop:
        LDA snakeBuf,X
        CMP snakeBuf
        BNE NextSeg
        LDA snakeBuf+1,X
        CMP snakeBuf+1
        BEQ GameOver               ; head hit this segment
NextSeg:
        INX
        INX
        CPX snakeLen
        BNE SelfLoop
        RTS



;────────────────────────────  A D V A N C E  ────────────────────────────────
;  1. Shift body pointers back 2 bytes
;  2. Add ±1 or ±32 to head pointer according to dirBits
AdvanceSnake:
        LDX snakeLen
        DEX
CopyLoop:
        LDA snakeBuf,X
        STA snakeBuf+2,X
        DEX
        BPL CopyLoop

        ; --- move head pointer ---
        LDA dirBits
        LSR                         ; Up?
        BCS MoveUp
        LSR                         ; Right?
        BCS MoveRight
        LSR                         ; Down?
        BCS MoveDown
        LSR                         ; Left?
        BCS MoveLeft
        RTS                         ; should never fall through

MoveUp:
        LDA snakeBuf
        SEC
        SBC #$20
        STA snakeBuf
        BCS UpOK                    ; same page?
        DEC snakeBuf+1
        LDA snakeBuf+1
        CMP #$01                    ; above row 0 → die
        BEQ GameOver
UpOK:
        JMP DrawSnake

MoveRight:
        INC snakeBuf
        LDA snakeBuf
        AND #$1F
        BEQ GameOver                ; wrapped 31→0
        JMP DrawSnake

MoveDown:
        LDA snakeBuf
        CLC
        ADC #$20
        STA snakeBuf
        BCC DownOK                  ; same page?
        INC snakeBuf+1
        LDA snakeBuf+1
        CMP #$06                    ; below row 31 → die
        BEQ GameOver
DownOK:
        JMP DrawSnake

MoveLeft:
        DEC snakeBuf
        LDA snakeBuf
        AND #$1F
        CMP #$1F                    ; wrapped 0→31?
        BEQ GameOver

;  Erase byte at (tail pointer) and write HEAD_CHAR at new head pointer.
DrawSnake:
        LDX snakeLen
        LDY #$00
        LDA #EMPTY_CHAR
        STA (snakeBuf,X)            ; erase tail

        LDX #$00
        LDA #HEAD_CHAR
        STA (snakeBuf,X)            ; draw head
        RTS

;────────────────────────────  D E L A Y  ─────────────────────────────────────
;  Burns  ⟨frameDelay⟩ × 1 279 cycles  (exact)  ➜ coarse, easy‑to‑tune delay.
;  inner loop  = 256 × (DEX 2c  + BNE‑taken 3c) – 1 final branch = 1 279 cycles
;  outer loop  = LDY n  + (inner + DEY/BNE)  ≈ 1 279 n  + 5 cycles overhead
Delay:
        LDY frameDelay        ; number of 1 279‑cycle “slices”
Outer:
        LDX #$00
Inner:
        DEX                   ; 2 cycles
        BNE Inner             ; 3 cycles taken, 2 not taken
        DEY                   ; 2 cycles
        BNE Outer             ; 3 cycles taken, 2 not taken
        RTS                   ; 6 cycles

;────────────────────────────  G A M E  O V E R  ─────────────────────────────
GameOver:
        JMP Freeze

Freeze:
        LDX #$00
FreezeSpin:
        NOP
        NOP
        DEX
        BNE FreezeSpin
        JMP Freeze                  ; endless

;───────────────────────────  V E C T O R S  ─────────────────────────────────
.segment "VECTORS"
        .word 0                     ; NMI
        .word Start                 ; RESET
        .word 0                     ; IRQ/BRK
