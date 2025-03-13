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
    else
        if {{if_check_statement}}; then
            \command prog "$@"
        else
            local result
            result="$(\command prog query -- "$@")" || return $?
            [[ -n "$result" ]] && __prog_cd "${result}"
        fi
    fi
}

function {{command}}() {
  __prog_p "$@"
}