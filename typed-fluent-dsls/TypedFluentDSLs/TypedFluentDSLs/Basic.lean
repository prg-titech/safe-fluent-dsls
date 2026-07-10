import Regex
import Init.Data.Array
-- Create a regex at compile-time using re! syntax
def dateRegexExample : Regex := re! r"\d{4}-\d{2}-\d{2}"

-- Find and return matches (and their positions as components of each `Substring`)
def allMatches := dateRegexExample.findAll
  "2025-05-24: Something happened\\n2025-05-26: Another thing happened"

-- #["2025-05-24", "2025-05-26"]
#eval allMatches.map (·.copy)

-- #[{ byteIdx := 0 }, { byteIdx := 32 }]
#eval allMatches.map (·.startInclusive.offset)

def matchesRegex (regex : Regex) (haystack : String) : Prop := regex.test haystack

instance {regex : Regex} {haystack : String} : Decidable (matchesRegex regex haystack) :=
  if h: regex.test haystack then isTrue h else isFalse h

@[ext]
structure Token (regex : Regex) where
  raw   : String
  valid : matchesRegex regex raw

def ID := Token (re! r"[a-zA-Z_$][a-zA-Z0-9_$]*")

instance : Inhabited (Token regex) where
  default := Token.mk "hello" sorry

-- elab "token!" regex:Regex lit:str : term => do
--  match Regex.parse lit.getString with
--  | Except.ok re => return toExpr re
--  | Except.error e => throwError s!"failed to parse regex: {e}"

namespace Token


def make! {regex : Regex} {α : Token regex} [i : Inhabited (Token regex)] (raw : String) [d : Decidable (matchesRegex regex raw)] : Token regex :=
  match d with
    | isTrue h => Token.mk raw h
    | isFalse _ => panic! s!"String {raw} does not match given regex."

end Token
