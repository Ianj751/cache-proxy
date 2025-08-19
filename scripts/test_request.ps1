# Script to send deterministic HTTP Requests.
# Headers like x-request-start are not pushed

$client = New-Object System.Net.Sockets.TcpClient("localhost", 8080)
$stream = $client.GetStream()
$writer = New-Object System.IO.StreamWriter($stream)
$reader = New-Object System.IO.StreamReader($stream)

# Send exactly the same HTTP request every time
$writer.WriteLine("GET / HTTP/1.1")
$writer.WriteLine("Host: postman-echo.com")
$writer.WriteLine("User-Agent: PowerShell")
$writer.WriteLine("Connection: close")
$writer.WriteLine("")
$writer.Flush()

# Read response
$response = ""
while ($line = $reader.ReadLine()) {
    $response += $line + "`n"
    if ($line -eq "") { break }  # End of headers
}
# Read body
while (!$reader.EndOfStream) {
    $response += $reader.ReadLine() + "`n"
}

Write-Output $response
$client.Close()