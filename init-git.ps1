# NetBoozt - Git Setup Script
# Inicializa el repositorio y hace el primer commit

Write-Host @"
╔══════════════════════════════════════════════════════════════╗
║              NetBoozt - Git Initialization                   ║
╚══════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan

$repoPath = "L:\NetworkFailover\NetBoozt"
$remoteUrl = "git@github.com:LOUST-PRO/NetBoozt_InternetUpgrade.git"

Write-Host "`n📦 Inicializando repositorio Git..." -ForegroundColor Yellow

cd $repoPath

# Verificar si ya existe .git
if (Test-Path ".git") {
    Write-Host "⚠️  .git ya existe. ¿Eliminar y reiniciar? (Y/N)" -ForegroundColor Yellow
    $response = Read-Host
    if ($response -eq "Y") {
        Remove-Item -Path ".git" -Recurse -Force
        Write-Host "✅ .git eliminado" -ForegroundColor Green
    } else {
        Write-Host "❌ Operación cancelada" -ForegroundColor Red
        exit
    }
}

# Inicializar Git
Write-Host "`n🔧 Ejecutando git init..." -ForegroundColor Cyan
git init

# Configurar usuario (opcional)
Write-Host "`n👤 Configurar usuario Git (opcional):" -ForegroundColor Cyan
$userName = Read-Host "Nombre (deja vacío para usar global)"
if ($userName) {
    git config user.name $userName
}

$userEmail = Read-Host "Email (deja vacío para usar global)"
if ($userEmail) {
    git config user.email $userEmail
}

# Crear .gitattributes
Write-Host "`n📝 Creando .gitattributes..." -ForegroundColor Cyan
@"
# Auto detect text files and perform LF normalization
* text=auto

# Explicitly declare files
*.py text
*.md text
*.txt text
*.json text
*.yaml text
*.yml text

# Denote binary files
*.png binary
*.jpg binary
*.ico binary
"@ | Out-File -FilePath ".gitattributes" -Encoding UTF8

# Agregar archivos
Write-Host "`n➕ Agregando archivos al staging..." -ForegroundColor Cyan
git add .

# Primer commit
Write-Host "`n💾 Creando primer commit..." -ForegroundColor Cyan
git commit -m "🚀 Initial commit - NetBoozt v1.0.0

- Windows network optimization module
- Modern GUI with ttkbootstrap
- 15+ TCP/IP optimizations
- Speedtest-cli integration
- Complete documentation
- Mermaid architecture diagrams
- Conservative/Balanced/Aggressive profiles

By LOUST (www.loust.pro)"

# Crear branch main
Write-Host "`n🌿 Creando branch 'main'..." -ForegroundColor Cyan
git branch -M main

# Agregar remote
Write-Host "`n🔗 Agregando remote 'origin'..." -ForegroundColor Cyan
git remote add origin $remoteUrl

Write-Host "`n✅ Repositorio inicializado correctamente!" -ForegroundColor Green
Write-Host "`n📤 Para hacer push:" -ForegroundColor Yellow
Write-Host "   git push -u origin main" -ForegroundColor White

Write-Host "`n📊 Estado del repositorio:" -ForegroundColor Cyan
git status

Write-Host "`n🎯 Siguiente paso:" -ForegroundColor Yellow
Write-Host "   cd $repoPath" -ForegroundColor White
Write-Host "   git push -u origin main" -ForegroundColor White
