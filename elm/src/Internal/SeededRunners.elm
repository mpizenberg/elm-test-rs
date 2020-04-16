module Internal.SeededRunners exposing (Kind(..), SeededRunners, fromTest, run)

import Array exposing (Array)
import ElmTestRs.Test.Result as TestResult exposing (TestResult)
import Random
import Test exposing (Test)
import Test.Runner exposing (Runner)


type alias SeededRunners =
    Result String { kind : Kind, runners : Array Runner }


type Kind
    = Plain
    | Only
    | Skipping



-- Functions


fromTest : Test -> { initialSeed : Int, fuzzRuns : Int } -> SeededRunners
fromTest masterTest { initialSeed, fuzzRuns } =
    case Test.Runner.fromTest fuzzRuns (Random.initialSeed initialSeed) masterTest of
        Test.Runner.Plain runnerList ->
            Ok { kind = Plain, runners = Array.fromList runnerList }

        Test.Runner.Only runnerList ->
            Ok { kind = Only, runners = Array.fromList runnerList }

        Test.Runner.Skipping runnerList ->
            Ok { kind = Skipping, runners = Array.fromList runnerList }

        Test.Runner.Invalid error ->
            Err error


run : Int -> Array Runner -> Maybe TestResult
run id runners =
    Array.get id runners
        |> Maybe.map (\runner -> { labels = runner.labels, expectations = runner.run () })
        |> Maybe.map (\result -> TestResult.fromExpectations result.labels result.expectations)
