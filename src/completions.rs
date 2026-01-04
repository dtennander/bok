/// Returns additional Fish shell completions for dynamic account and ref completions
pub fn fish_dynamic_completions() -> &'static str {
    r#"
# Dynamic completion helpers for bok

# Helper function to get the positional argument count for a subcommand
function __fish_bok_positional_count
    set -l cmd (commandline -opc)
    set -e cmd[1] # Remove 'bok'
    set -e cmd[1] # Remove subcommand
    # Filter out options (starting with -)
    set -l count 0
    for arg in $cmd
        if not string match -q -- '-*' $arg
            set count (math $count + 1)
        end
    end
    echo $count
end

# Dynamic completion for accounts
function __fish_bok_complete_accounts
    bok accounts 2>/dev/null
end

# Dynamic completion for refs
function __fish_bok_complete_refs
    bok log 2>/dev/null
end

# record/rec subcommand - dynamic account completions for debit and credit
complete -c bok -n "__fish_bok_using_subcommand record; and test (__fish_bok_positional_count) -lt 2" -f -a "(__fish_bok_complete_accounts)"
complete -c bok -n "__fish_bok_using_subcommand rec; and test (__fish_bok_positional_count) -lt 2" -f -a "(__fish_bok_complete_accounts)"

# show subcommand - dynamic ref completions
complete -c bok -n "__fish_bok_using_subcommand show" -f -a "(__fish_bok_complete_refs)"
complete -c bok -n "__fish_bok_using_subcommand show" -f -a "HEAD" -d 'Current head entry'

# log subcommand - dynamic ref completions
complete -c bok -n "__fish_bok_using_subcommand log" -f -a "(__fish_bok_complete_refs)"
complete -c bok -n "__fish_bok_using_subcommand log" -f -a "HEAD" -d 'Current head entry'
"#
}

/// Returns additional Bash shell completions for dynamic account and ref completions
pub fn bash_dynamic_completions() -> &'static str {
    r#"
# Dynamic completion helpers for bok

_bok_complete_accounts() {
    local accounts
    accounts=$(bok accounts 2>/dev/null | cut -f1)
    COMPREPLY=($(compgen -W "$accounts" -- "${COMP_WORDS[COMP_CWORD]}"))
}

_bok_complete_refs() {
    local refs
    refs=$(bok log 2>/dev/null | cut -f1)
    refs="$refs HEAD"
    COMPREPLY=($(compgen -W "$refs" -- "${COMP_WORDS[COMP_CWORD]}"))
}

# Override completions for record/rec subcommand
_bok_record() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Count positional arguments (excluding options)
    local pos_count=0
    for ((i=2; i<COMP_CWORD; i++)); do
        if [[ "${COMP_WORDS[i]}" != -* ]]; then
            ((pos_count++))
        fi
    done

    # First two positional args are debit and credit accounts
    if [[ $pos_count -lt 2 ]]; then
        _bok_complete_accounts
        return
    fi
}

# Override completions for show subcommand
_bok_show() {
    _bok_complete_refs
}

# Override completions for log subcommand
_bok_log() {
    _bok_complete_refs
}

# Hook into the main completion function
if [[ $(type -t _bok) == function ]]; then
    _bok_orig=$(declare -f _bok)
    eval "_bok_base${_bok_orig#_bok}"

    _bok() {
        local cur prev words cword
        _init_completion || return

        if [[ ${words[1]} == "record" || ${words[1]} == "rec" ]]; then
            _bok_record
        elif [[ ${words[1]} == "show" ]]; then
            _bok_show
        elif [[ ${words[1]} == "log" ]]; then
            _bok_log
        else
            _bok_base
        fi
    }
fi
"#
}

/// Returns additional Zsh shell completions for dynamic account and ref completions
pub fn zsh_dynamic_completions() -> &'static str {
    r#"
# Dynamic completion helpers for bok

_bok_complete_accounts() {
    local -a accounts
    accounts=(${(f)"$(bok accounts 2>/dev/null)"})
    _describe -t accounts 'accounts' accounts
}

_bok_complete_refs() {
    local -a refs
    refs=(${(f)"$(bok log 2>/dev/null)"})
    refs+=("HEAD:Current head entry")
    _describe -t refs 'refs' refs
}

# Add dynamic completions to existing bok completion
if (( $+functions[_bok] )); then
    _bok_dynamic() {
        case "$words[2]" in
            record|rec)
                # Count positional arguments
                local pos_count=0
                for ((i=3; i<CURRENT; i++)); do
                    if [[ "${words[i]}" != -* ]]; then
                        ((pos_count++))
                    fi
                done
                if (( pos_count < 2 )); then
                    _bok_complete_accounts
                    return
                fi
                ;;
            show|log)
                _bok_complete_refs
                return
                ;;
        esac
        _bok "$@"
    }
    compdef _bok_dynamic bok
fi
"#
}
