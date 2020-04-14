module ElmTestRs.Test.Reporter.Junit exposing (implementation)

import Array exposing (Array)
import Dict
import ElmTestRs.Test.Reporter.Interface exposing (Interface)
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Xml
import Xml.Encode as Encode


implementation : Interface
implementation =
    { onBegin = always (Just "Begin JUNIT report\n")
    , onResult = always Nothing
    , onEnd = \results -> Just (summary results)
    }


summary : Array TestResult -> String
summary results =
    let
        nbTests =
            Array.length results

        ( nbFailures, nbSkipped, duration ) =
            Array.foldl accStats ( 0, 0, 0 ) results

        encodedTests =
            Array.toList results
                |> List.map encodeTestResult
                |> Encode.list

        suiteAttributes =
            Dict.fromList
                [ ( "name", Encode.string "elm-test-rs" )
                , ( "tests", Encode.int nbTests )
                , ( "failures", Encode.int nbFailures )
                , ( "skipped", Encode.int nbSkipped )
                , ( "time", Encode.float duration )
                ]
    in
    Encode.encode 0 <|
        Encode.list
            [ Encode.string "<?xml version=\"1.0\"?>"
            , Encode.object [ ( "testsuite", suiteAttributes, encodedTests ) ]
            ]


{-| TODO: I don't know if we should count 1 value per "TestResult"
or if all outcomes (if multiples) of a test should count.
In the later case, we should count the number Passed outcomes.
The annoyingt thing is that passed + todos + failures >= "nbTests"
in that case so it might be weird to report it that way.

Currently I count 1 value per "TestResult" which is either pass or fail.
So the "nbSkipped" count does not grow.

-}
accStats : TestResult -> ( Int, Int, Float ) -> ( Int, Int, Float )
accStats result ( nbFailures, nbSkipped, duration ) =
    case result of
        TestResult.Passed passed ->
            ( nbFailures, nbSkipped, duration + passed.duration )

        TestResult.Failed failure ->
            ( nbFailures + 1, nbSkipped, duration + failure.duration )


encodeTestResult : TestResult -> Xml.Value
encodeTestResult result =
    let
        ( labels, duration, failures ) =
            case result of
                TestResult.Passed test ->
                    ( test.labels, test.duration, Encode.null )

                TestResult.Failed test ->
                    ( test.labels, test.duration, encodeFailures )

        ( class, name ) =
            classAndName labels

        attributesDict =
            Dict.fromList
                [ ( "classname", Encode.string class )
                , ( "name", Encode.string name )
                , ( "time", Encode.float duration )
                ]
    in
    Encode.object
        [ ( "testcase", attributesDict, failures ) ]


classAndName : List String -> ( String, String )
classAndName labels =
    classAndNameHelper "" labels []


classAndNameHelper : String -> List String -> List String -> ( String, String )
classAndNameHelper defaultName labels accLabels =
    case labels of
        [] ->
            ( "", defaultName )

        name :: [] ->
            ( String.join " " (List.reverse accLabels), name )

        class :: otherLabels ->
            classAndNameHelper defaultName otherLabels (class :: accLabels)


encodeFailures : Xml.Value
encodeFailures =
    Encode.object [ ( "failure", Dict.empty, Encode.null ) ]
