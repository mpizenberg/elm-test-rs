module ElmTestRs.Test.Result exposing (Outcome(..), TestResult, decoder, encode, fromExpectations)

import ElmTestRs.Test.Failure as Failure exposing (Failure)
import Expect exposing (Expectation)
import Json.Decode as Decode exposing (Decoder, Value)
import Json.Encode as Encode
import Test.Runner



-- Types


type alias TestResult =
    { labels : List String
    , outcomes : List Outcome
    , duration : Float
    }


{-| Real outcome will actually be more complex
-}
type Outcome
    = Passed
    | Todo String
    | Failed Failure



-- Functions


fromExpectations : List String -> List Expectation -> TestResult
fromExpectations labels expectations =
    { labels = labels
    , outcomes = outcomesFromExpectations expectations

    -- TODO: deal with duration yet
    , duration = 0
    }


outcomesFromExpectations : List Expectation -> List Outcome
outcomesFromExpectations expectations =
    List.foldl outcomesBuilder [] expectations


outcomesBuilder : Expectation -> List Outcome -> List Outcome
outcomesBuilder expectation outcomes =
    case Test.Runner.getFailureReason expectation of
        Nothing ->
            outcomes

        Just failure ->
            if Test.Runner.isTodo expectation then
                Todo failure.description :: outcomes

            else
                Failed failure :: outcomes



-- Encode


encode : TestResult -> Value
encode result =
    Encode.object
        [ ( "labels", Encode.list Encode.string result.labels )
        , ( "outcomes", Encode.list encodeOutcome result.outcomes )
        , ( "duration", Encode.float result.duration )
        ]


encodeOutcome : Outcome -> Value
encodeOutcome outcome =
    case outcome of
        Passed ->
            Encode.object [ ( "variant", Encode.string "Passed" ) ]

        Todo todo ->
            Encode.object
                [ ( "variant", Encode.string "Todo" )
                , ( "todo", Encode.string todo )
                ]

        Failed failure ->
            Encode.object
                [ ( "variant", Encode.string "Failed" )
                , ( "failure", Failure.encode failure )
                ]



-- Decoder


decoder : Decoder TestResult
decoder =
    Decode.map3 TestResult
        (Decode.field "labels" <| Decode.list Decode.string)
        (Decode.field "outcome" <| Decode.list outcomeDecoder)
        (Decode.field "duration" Decode.float)


outcomeDecoder : Decoder Outcome
outcomeDecoder =
    Decode.field "variant" Decode.string
        |> Decode.andThen decodeOutcomeHelp


decodeOutcomeHelp : String -> Decoder Outcome
decodeOutcomeHelp constructor =
    case constructor of
        "Passed" ->
            Decode.succeed Passed

        "Todo" ->
            Decode.map Todo (Decode.field "todo" Decode.string)

        "Failed" ->
            Decode.map Failed (Decode.field "failure" Failure.decoder)

        other ->
            Decode.fail <| "Unknown constructor for type Outcome: " ++ other
