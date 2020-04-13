module Runner exposing (main)

{{ user_imports }}
import Test
import ElmTestRs.Test.Runner

main : ElmTestRs.Test.Runner.Program
main =
    [ {{ tests }} ]
        |> Test.concat
        |> ElmTestRs.Test.Runner.start
