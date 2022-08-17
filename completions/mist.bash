_mist_get_pkglist() {
    mapfile -t COMPREPLY < <("${words[0]}" quick-list "${@}")
}

_mist_gen_compreply() {
    mapfile -t COMPREPLY < <(compgen -W "${1}" -- "${2}")
}

_mist_pkg_specified_check() {
    if [[ "${#nonopts[@]}"  -gt 3 ]]; then
        _mist_gen_compreply '${opts[@]}' "${cur}"
    else
        _mist_get_pkglist "${@}"
    fi
}

_mist() {
    local cur prev words cword
    _init_completion || return

    local cmds=(
        'clone'
        'comment'
        'help'
        'list-comments'
        'remove'
        'search'
        'update'
        'whoami'
    )

    # Get a list of arguments that are nonoptions.
    mapfile -t nonopts < <(printf '%s\n' "${words[@]}" | grep -v '^-')

    if [[ "${#words[@]}" == 2 ]]; then
        mapfile -t COMPREPLY < <(compgen -W '${cmds[@]}' "${cur}")
        return
    fi

    case "${nonopts[1]}" in
        clone)
        opts=('--mpr-url')

            case "${prev}" in
                --mpr-url)
                    return
                    ;;
            esac

            case "${cur}" in
                -*)
                    _mist_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mist_pkg_specified_check "${cur}"
                    return
                    ;;
            esac
            ;;
        comment)
            opts=('--mpr-url' '--msg' '--token')

            case "${prev}" in
                --token|--mpr-url|--msg)
                    return
                    ;;
            esac

            case "${cur}" in
                -*)
                    _mist_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mist_pkg_specified_check "${cur}"
                    return
                    ;;
            esac
            ;;
        help)
            return
            ;;
        list-comments)
            opts=('--mpr-url' '--paging')

            case "${prev}" in
                --mpr-url)
                    return
                    ;;
                --paging)
                    opts=('auto' 'never' 'always')
                    _mist_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
            esac
            
            case "${cur}" in
                -*)
                    _mist_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mist_pkg_specified_check "${cur}"
                    return
                    ;;
            esac
            ;;
        remove)
            opts=('--autoremove' '--purge')

            case "${cur}" in
            -*)
                _mist_gen_compreply '${opts[@]}' "${cur}"
                return
                ;;
            *)
                _mist_get_pkglist '--apt-only' "${cur}"
                return
                ;;
            esac
            ;;

        search|list)
            opts=('--mpr-url' '--apt-only' '--mpr-only' '--name-only' '--installed')

            case "${prev}" in
                --mpr-url)
                    return
                    ;;
            esac

            case "${cur}" in
                -*)
                    _mist_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mist_pkg_specified_check "${cur}"
                    return
                    ;;
            esac
            ;;
        whoami)
        opts=('--token' '--mpr-url')

            case "${prev}" in
                --token|--mpr-url)
                    return
                    ;;
            esac

            _mist_gen_compreply '${opts[@]}' "${cur}"
            return
            ;;
    esac
}

complete -F _mist mist
# vim: set sw=4 expandtab:
