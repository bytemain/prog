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
        \command prog "$@" || return $?
        # Extract repo name from arguments (find first non-flag argument after 'add')
        local url=""
        local skip_next=false
        shift  # skip 'add'
        for arg in "$@"; do
            if $skip_next; then
                skip_next=false
                continue
            fi
            # Skip flags and their values
            if [[ "$arg" == --* ]]; then
                continue
            elif [[ "$arg" == -* ]]; then
                # Single-letter flags might have values
                skip_next=true
                continue
            else
                url="$arg"
                break
            fi
        done
        if [[ -n "$url" ]]; then
            local repo_name="${url##*/}"
            repo_name="${repo_name%.git}"
            if [[ -n "$repo_name" ]]; then
                local result
                result="$(\command prog find --query -- "$repo_name")" || return $?
                [[ -n "$result" ]] && __prog_cd "${result}"
            fi
        fi
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