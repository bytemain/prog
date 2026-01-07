function __prog_cd {
    param([string]$path)
    Set-Location -Path $path
}

function __prog_p {
    param(
        [Parameter(ValueFromRemainingArguments = $true)]
        $args
    )

    if ($args.Count -eq 0) {
        prog
    }
    elseif ($args.Count -eq 1 -and (
            (Test-Path -Path $args[0] -PathType Container) -or 
            ($args[0] -eq '-') -or 
            ($args[0] -match '^[-+][0-9]$')
        )) {
        __prog_cd $args[0]
    }
    elseif ($args.Count -eq 2 -and $args[0] -eq '--') {
        prog $args[1]
    }
    elseif ($args[0] -eq 'add') {
        prog @args
        if ($LASTEXITCODE -ne 0) {
            return $LASTEXITCODE
        }
        
        # Extract repo name from arguments (find first non-flag argument after 'add')
        $url = $null
        $skipNext = $false
        for ($i = 1; $i -lt $args.Count; $i++) {
            $arg = $args[$i]
            if ($skipNext) {
                $skipNext = $false
                continue
            }
            # Skip flags and their values
            if ($arg.StartsWith('--')) {
                continue
            }
            elseif ($arg.StartsWith('-')) {
                # Single-letter flags might have values
                $skipNext = $true
                continue
            }
            else {
                $url = $arg
                break
            }
        }
        
        if ($url) {
            $repoName = ($url -split '/')[-1] -replace '\.git$', ''
            
            if ($repoName) {
                $result = $null
                try {
                    $result = prog find --query -- $repoName
                    if ($LASTEXITCODE -ne 0) {
                        return $LASTEXITCODE
                    }
                }
                catch {
                    return 1
                }
                
                if ($result) {
                    __prog_cd $result
                }
            }
        }
    }
    else {
        # {{if_check_statement}} would be replaced during template processing
        if ({{if_check_statement}}) {
            prog @args
        }
        else {
            $result = $null
            try {
                $result = prog find --query -- @args
                if ($LASTEXITCODE -ne 0) {
                    return $LASTEXITCODE
                }
            }
            catch {
                return 1
            }
            
            if ($result) {
                __prog_cd $result
            }
        }
    }
}

function {{command}} {
    __prog_p @args
}