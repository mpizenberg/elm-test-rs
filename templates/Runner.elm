module Runner exposing (main)

{{ user_imports }}
import Test
import ElmTestRs.Runner

main : ElmTestRs.Runner.Program
main =
    [ {{ tests }} ]
        |> Test.concat
        |> ElmTestRs.Runner.run {{ flags }}
