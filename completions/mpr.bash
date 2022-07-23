_mpr_get_pkglist() {
    mapfile -t opts < <("${words[0]}" pkglist)
}

_mpr_gen_compreply() {
    mapfile -t COMPREPLY < <(compgen -W "${1}" -- "${2}")
}

_mpr_pkg_specified_check() {
    if [[ "${#nonopts[@]}"  -gt 3 ]]; then
        _mpr_gen_compreply '${opts[@]}' "${cur}"
    else
        _mpr_get_pkglist
        _mpr_gen_compreply '${opts[@]}' "${cur}"
    fi
}

_mpr() {
    local cur prev words cword
    _init_completion || return

    local cmds=(
        'clone'
        'comment'
        'help'
        'info'
        'list-comments'
        'search'
        'update'
        'whoami'
    )
    local opts=(
        '--mpr-url'
        '--token'
    )
    
    # Get a list of arguments that are nonoptions.
    mapfile -t nonopts < <(printf '%s\n' "${words[@]}" | grep -v '^-')

    if [[ "${#words[@]}" == 2 ]]; then
        mapfile -t COMPREPLY < <(compgen -W '${cmds[@]}' "${cur}")
        return
    fi

    case "${nonopts[1]}" in
        clone|info)
            case "${prev}" in
                --token|--mpr-url)
                    return
                    ;;
            esac

            case "${cur}" in
                -*)
                    _mpr_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mpr_pkg_specified_check
                    return
                    ;;
            esac
            ;;
        comment)
            case "${prev}" in
                --token|--mpr-url|--msg)
                    return
                    ;;
            esac

            opts+=('--msg')
            case "${cur}" in
                -*)
                    _mpr_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mpr_pkg_specified_check
                    return
                    ;;
            esac
            ;;
        help)
            return
            ;;
        list-comments)
            case "${prev}" in
                --token|--mpr-url)
                    return
                    ;;
                --paging)
                    opts=('auto' 'never' 'always')
                    _mpr_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
            esac
            
            opts+=('--paging')
            case "${cur}" in
                -*)
                    _mpr_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mpr_pkg_specified_check
                    return
                    ;;
            esac
            ;;
        search)
            case "${prev}" in
                --token|--mpr-url)
                    return
                    ;;
            esac

            opts+=('--apt-only' '--mpr-only')
            case "${cur}" in
                -*)
                    _mpr_gen_compreply '${opts[@]}' "${cur}"
                    return
                    ;;
                *)
                    _mpr_pkg_specified_check
                    return
                    ;;
            esac
            ;;
        update)
            case "${prev}" in
                --token|--mpr-url)
                    return
                    ;;
            esac

            _mpr_gen_compreply '${opts[@]}' "${cur}"
            ;;
        whoami)
            case "${prev}" in
                --token|--mpr-url)
                    return
                    ;;
            esac

            _mpr_gen_compreply '${opts[@]}' "${cur}"
            return
            ;;
    esac
}

complete -F _mpr mpr
# vim: set sw=4 expandtab:
