Add-Type -AssemblyName System.Drawing

$iconDir = 'D:/project/react/excel/src-tauri/icons'

# Create bitmap images
$sizes = @(16, 32, 48, 256)
$bitmaps = @{}

foreach ($size in $sizes) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

    # Blue background
    $g.Clear([System.Drawing.Color]::FromArgb(74, 158, 255))

    # White circle
    $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $padding = [int]($size * 0.1)
    $g.FillEllipse($brush, $padding, $padding, $size - 2*$padding, $size - 2*$padding)

    # Blue X
    $fontSize = [float]($size * 0.5)
    $font = New-Object System.Drawing.Font('Arial', $fontSize, [System.Drawing.FontStyle]::Bold)
    $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(74, 158, 255))
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $rect = New-Object System.Drawing.RectangleF(0, 0, $size, $size)
    $g.DrawString('X', $font, $textBrush, $rect, $sf)

    $g.Dispose()
    $bitmaps[$size] = $bmp
}

# Save PNG files
$bitmaps[32].Save("$iconDir/32x32.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp128 = New-Object System.Drawing.Bitmap($bitmaps[256], 128, 128)
$bmp128.Save("$iconDir/128x128.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp256 = New-Object System.Drawing.Bitmap($bitmaps[256], 256, 256)
$bmp256.Save("$iconDir/128x128@2x.png", [System.Drawing.Imaging.ImageFormat]::Png)

# Create ICO file manually with proper format
$icoPath = "$iconDir/icon.ico"
$fs = [System.IO.File]::Create($icoPath)
$bw = New-Object System.IO.BinaryWriter($fs)

# ICO Header
$bw.Write([Int16]0)       # Reserved
$bw.Write([Int16]1)       # Type (1 = ICO)
$bw.Write([Int16]3)       # Number of images

# Calculate offsets
$headerSize = 6 + (16 * 3)
$offset = $headerSize

# PNG data for each size
$pngData = @()
foreach ($size in @(16, 32, 48)) {
    $ms = New-Object System.IO.MemoryStream
    $bitmaps[$size].Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $pngData += ,$ms.ToArray()
    $ms.Dispose()
}

# Directory entries
for ($i = 0; $i -lt 3; $i++) {
    $size = @(16, 32, 48)[$i]
    $data = $pngData[$i]

    $bw.Write([byte]$(if ($size -eq 256) { 0 } else { $size }))  # Width
    $bw.Write([byte]$(if ($size -eq 256) { 0 } else { $size }))  # Height
    $bw.Write([byte]0)       # Color palette
    $bw.Write([byte]0)       # Reserved
    $bw.Write([Int16]1)      # Color planes
    $bw.Write([Int16]32)     # Bits per pixel
    $bw.Write([Int32]$data.Length)  # Image size
    $bw.Write([Int32]$offset)       # Offset

    $offset += $data.Length
}

# Write image data
foreach ($data in $pngData) {
    $bw.Write($data)
}

$bw.Close()
$fs.Close()

# Cleanup
foreach ($bmp in $bitmaps.Values) { $bmp.Dispose() }
$bmp128.Dispose()
$bmp256.Dispose()

Write-Host "Icons created successfully"
