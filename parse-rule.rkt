; The rule parser written in Racket for no good reason :)
;
; Libraries: `raco pkg install megaparsack-lib`

#lang racket

(require megaparsack megaparsack/text)
(require data/monad)
(require data/applicative)
(require data/functor)

(define range/p
  (do [l <- integer/p]
    (string/p "-")
    [r <- integer/p]
    (pure (list l r))))

(define (bit-at m)
  (arithmetic-shift 1 m))

(define (range->bitmask range)
  (bitwise-xor (- (bit-at (first range)) 1)
               (- (bit-at (+ (second range) 1)) 1)))

(define bitmask/p
  (map (curry foldl bitwise-ior 0)
       (many/p (or/p (try/p (map range->bitmask range/p))
                     (map bit-at integer/p)) #:sep (string/p ","))))

(define rule/p
  (do [x <- bitmask/p]
    (string/p "/")
    [y <- bitmask/p]
    (string/p "/")
    [z <- bitmask/p]
    (string/p "/")
    [w <- (or/p (map (const #t) (string/p "M"))
                (map (const #f) (string/p "N")))]
    (pure (list x y z w))))