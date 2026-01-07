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
        $restArgs = $args[1..($args.Count - 1)]
        $output = $null
        try {
            $output = prog add --cd -- @restArgs
            if ($LASTEXITCODE -ne 0) {
                return $LASTEXITCODE
            }
        }
        catch {
            return 1
        }
        
        if ($output) {
            # Print the output and get the last line as the path for cd
            Write-Output $output
            $lines = $output -split "`n"
            $result = $lines[-1].Trim()
            if ($result -and (Test-Path -Path $result -PathType Container)) {
                __prog_cd $result
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