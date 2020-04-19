module ElmTestRunner.Result exposing
    ( TestResult(..), fromExpectations, encode, decoder
    , Summary, summary
    )

{-| Types and functions to manipulate a test result.


# Manipulation of the result of a test run

@docs TestResult, fromExpectations, encode, decoder


# Helper functions

@docs Summary, summary

-}

import Array exposing (Array)
import ElmTestRunner.Failure as Failure exposing (Failure)
import Expect exposing (Expectation)
import Json.Decode as Decode exposing (Decoder, Value)
import Json.Encode as Encode
import Test.Runner


{-| Type summarizing the results of a test run.
It is obtained from the list of expectations returned by calling runner.run ().
-}
type TestResult
    = Passed { labels : List String, duration : Float }
    | Failed { labels : List String, duration : Float, todos : List String, failures : List Failure }


{-| Convert a list of expectations (results of a run) into a `TestResult`.
Return the `Failed` variant if there is any todo or failure in the expectations.
-}
fromExpectations : List String -> List Expectation -> TestResult
fromExpectations labels expectations =
    case failuresAndTodos expectations of
        ( [], [] ) ->
            Passed { labels = labels, duration = 0 }

        ( todos, failures ) ->
            Failed { labels = labels, duration = 0, todos = todos, failures = failures }


failuresAndTodos : List Expectation -> ( List String, List Failure )
failuresAndTodos expectations =
    List.foldl accumFailuresAndTodos ( [], [] ) expectations


accumFailuresAndTodos : Expectation -> ( List String, List Failure ) -> ( List String, List Failure )
accumFailuresAndTodos expectation (( todos, failures ) as outcomes) =
    case Test.Runner.getFailureReason expectation of
        Nothing ->
            outcomes

        Just failure ->
            if Test.Runner.isTodo expectation then
                ( failure.description :: todos, failures )

            else
                ( todos, failure :: failures )


{-| Encode a `TestResult`.
-}
encode : TestResult -> Value
encode =
    encodeTestResult


{-| Decode a `TestResult`.
-}
decoder : Decoder TestResult
decoder =
    decodeTestResult


{-| Quantitative summary of all test results.
-}
type alias Summary =
    { totalDuration : Float, nbPassed : Int, nbFailed : Int }


{-| Report a quantitative summary of test results.
-}
summary : Array TestResult -> Summary
summary =
    Array.foldl accumStats { totalDuration = 0, nbPassed = 0, nbFailed = 0 }


accumStats : TestResult -> Summary -> Summary
accumStats result { totalDuration, nbPassed, nbFailed } =
    case result of
        Passed { duration } ->
            { totalDuration = totalDuration + duration
            , nbPassed = nbPassed + 1
            , nbFailed = nbFailed
            }

        Failed { duration } ->
            { totalDuration = totalDuration + duration
            , nbPassed = nbPassed
            , nbFailed = nbFailed + 1
            }



-- Functions needed by the automatically generated decoders


decodeFailure =
    Failure.decoder


encodeFailure =
    Failure.encode



-- Automatically generated decoders and encoders with https://dkodaj.github.io/decgen/


type alias Record_labels_ListString_duration_Float_todos_ListString_failures_ListFailure_ =
    { labels : List String, duration : Float, todos : List String, failures : List Failure }


type alias Record_labels_ListString_duration_Float_ =
    { labels : List String, duration : Float }


decodeRecord_labels_ListString_duration_Float_ =
    Decode.map2
        Record_labels_ListString_duration_Float_
        (Decode.field "labels" (Decode.list Decode.string))
        (Decode.field "duration" Decode.float)


decodeRecord_labels_ListString_duration_Float_todos_ListString_failures_ListFailure_ =
    Decode.map4
        Record_labels_ListString_duration_Float_todos_ListString_failures_ListFailure_
        (Decode.field "labels" (Decode.list Decode.string))
        (Decode.field "duration" Decode.float)
        (Decode.field "todos" (Decode.list Decode.string))
        (Decode.field "failures" (Decode.list decodeFailure))


decodeTestResult =
    Decode.field "Constructor" Decode.string |> Decode.andThen decodeTestResultHelp


decodeTestResultHelp constructor =
    case constructor of
        "Passed" ->
            Decode.map
                Passed
                (Decode.field "A1" decodeRecord_labels_ListString_duration_Float_)

        "Failed" ->
            Decode.map
                Failed
                (Decode.field "A1" decodeRecord_labels_ListString_duration_Float_todos_ListString_failures_ListFailure_)

        other ->
            Decode.fail <| "Unknown constructor for type TestResult: " ++ other


encodeRecord_labels_ListString_duration_Float_ a =
    Encode.object
        [ ( "labels", Encode.list Encode.string a.labels )
        , ( "duration", Encode.float a.duration )
        ]


encodeRecord_labels_ListString_duration_Float_todos_ListString_failures_ListFailure_ a =
    Encode.object
        [ ( "labels", Encode.list Encode.string a.labels )
        , ( "duration", Encode.float a.duration )
        , ( "todos", Encode.list Encode.string a.todos )
        , ( "failures", Encode.list encodeFailure a.failures )
        ]


encodeTestResult a =
    case a of
        Passed a1 ->
            Encode.object
                [ ( "Constructor", Encode.string "Passed" )
                , ( "A1", encodeRecord_labels_ListString_duration_Float_ a1 )
                ]

        Failed a1 ->
            Encode.object
                [ ( "Constructor", Encode.string "Failed" )
                , ( "A1", encodeRecord_labels_ListString_duration_Float_todos_ListString_failures_ListFailure_ a1 )
                ]
