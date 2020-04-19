module ElmTestRunner.Log exposing (capture, parser, seedPattern)

import Parser exposing ((|.), (|=), Parser)
import Random


capture : { seed : Int, id : Int } -> (a -> b) -> a -> b
capture { seed, id } f a =
    let
        id_ =
            Debug.log (seedPattern seed) id
    in
    always (f a) id_


seedPattern : Int -> String
seedPattern seed =
    let
        initialSeed =
            Random.initialSeed seed
    in
    Random.step (Random.list 32 (Random.int 0 9)) initialSeed
        |> Tuple.first
        |> List.map String.fromInt
        |> String.join ""


parser : String -> Parser (List { id : Int, logs : String })
parser pattern =
    Parser.sequence
        { start = pattern ++ ": "
        , separator = pattern ++ ": "
        , end = ""
        , spaces = Parser.spaces
        , item = parseOneLog pattern
        , trailing = Parser.Optional
        }


parseOneLog : String -> Parser { id : Int, logs : String }
parseOneLog pattern =
    Parser.succeed (\id logs -> { id = id, logs = logs })
        |= Parser.int
        |. Parser.spaces
        -- Beware that the last item will get the last \n before end
        -- in the chomped string. Not sure of how to remove it
        -- without removing potential intentional \n
        |= Parser.getChompedString (Parser.chompUntilEndOr ("\n" ++ pattern))
