module Reporter.Junit exposing (implementation)

import Array exposing (Array)
import Data
import Dict
import Reporter.Interface exposing (Interface)
import Xml
import Xml.Encode as Encode


implementation : Interface
implementation =
    { onBegin = always (Just "Begin JUNIT report\n")
    , onResult = always Nothing
    , onEnd = \results -> Just (summary results)
    }


summary : Array Data.TestResult -> String
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


accStats : Data.TestResult -> ( Int, Int, Float ) -> ( Int, Int, Float )
accStats result ( nbFailures, nbSkipped, duration ) =
    case result.outcome of
        Data.Passed ->
            ( nbFailures, nbSkipped, duration + result.duration )

        Data.Todo ->
            ( nbFailures + 1, nbSkipped, duration + result.duration )

        Data.Failed ->
            ( nbFailures + 1, nbSkipped, duration + result.duration )


encodeTestResult : Data.TestResult -> Xml.Value
encodeTestResult result =
    let
        ( class, name ) =
            classAndName result.labels

        attributesDict =
            Dict.fromList
                [ ( "classname", Encode.string class )
                , ( "name", Encode.string name )
                , ( "time", Encode.float result.duration )
                ]

        failures =
            case result.outcome of
                Data.Passed ->
                    Encode.null

                Data.Todo ->
                    encodeFailures

                Data.Failed ->
                    encodeFailures
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
