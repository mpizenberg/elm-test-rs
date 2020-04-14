module ElmTestRs.Test.Result exposing (TestResult(..), decoder, encode, fromExpectations)

import ElmTestRs.Test.Failure as Failure exposing (Failure)
import Expect exposing (Expectation)
import Json.Decode as Decode
import Json.Encode as Encode
import Test.Runner



-- Types


type TestResult
    = Passed { labels : List String, duration : Float }
    | Failed { labels : List String, duration : Float, todos : List String, failures : List Failure }



-- Functions


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
        (Decode.field "failures" (Decode.list Failure.decoder))


decoder =
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
        , ( "failures", Encode.list Failure.encode a.failures )
        ]


encode a =
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
