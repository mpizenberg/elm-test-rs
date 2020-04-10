module Data exposing (Outcome(..), TestResult, encodeTestResult, testResultDecoder)

import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode


type alias TestResult =
    { labels : List String
    , outcome : Outcome
    , duration : Float
    }


{-| Real outcomes will actually be more complex
-}
type Outcome
    = Passed
    | Todo
    | Failed



-- Encode


encodeTestResult : TestResult -> Encode.Value
encodeTestResult result =
    Encode.object
        [ ( "labels", Encode.list Encode.string result.labels )
        , ( "outcome", encodeOutcome result.outcome )
        , ( "duration", Encode.float result.duration )
        ]


encodeOutcome : Outcome -> Encode.Value
encodeOutcome outcome =
    case outcome of
        Passed ->
            Encode.string "passed"

        Todo ->
            Encode.string "todo"

        Failed ->
            Encode.string "failed"



-- Decoder


testResultDecoder : Decoder TestResult
testResultDecoder =
    Decode.map3 TestResult
        (Decode.field "labels" <| Decode.list Decode.string)
        (Decode.field "outcome" outcomeDecoder)
        (Decode.field "duration" Decode.float)


outcomeDecoder : Decoder Outcome
outcomeDecoder =
    Decode.map outcomeFromString Decode.string


{-| Will actually not be a string for "real" outcomes
-}
outcomeFromString : String -> Outcome
outcomeFromString str =
    case str of
        "todo" ->
            Todo

        "failed" ->
            Failed

        _ ->
            Passed
