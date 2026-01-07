function __prog_cd() {
    \builtin cd -- "$@"
}

function __prog_p() {
    if [[ "$#" -eq 0 ]]; then
        \command prog
    elif [[ "$#" -eq 1 ]] && {
            [[ -d "$1" ]] || [[ "$1" = '-' ]] || [[ "$1" =~ ^[-+][0-9]$ ]]
        }; then
        __prog_cd "$1"
    elif [[ "$#" -eq 2 ]] && [[ "$1" = "--" ]]; then
        \command prog "$2"
    elif [[ "$1" = "add" ]]; then
        shift
        local output
        output="$(\command prog add --cd -- "$@")" || return $?
        echo "$output"
        # Get the last line as the path for cd
        local result
        result="$(echo "$output" | tail -n 1)"
        [[ -n "$result" ]] && [[ -d "$result" ]] && __prog_cd "${result}"
    else
        if {{if_check_statement}}; then
            \command prog "$@"
        else
            local result
            result="$(\command prog find --query -- "$@")" || return $?
            [[ -n "$result" ]] && __prog_cd "${result}"
        fi
    fi
}

function {{command}}() {
  __prog_p "$@"
}