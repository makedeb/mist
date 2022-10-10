@test "run update" {
    target/debug/mist update
}

@test "search with no results" {
    target/debug/mist search 'nonexistent'
}

@test "list with no results" {
    target/debug/mist list 'nonexistent'
}