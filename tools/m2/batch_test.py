#!/usr/bin/env python3
"""
Batch testing tool for M2 parser - quickly validate multiple models.
"""

import click
from pathlib import Path
from rich.console import Console
from rich.table import Table
from parser import ComprehensiveM2Parser

console = Console()

def test_model(file_path: str) -> dict:
    """Test parse a single M2 model and return summary"""
    parser = ComprehensiveM2Parser(file_path)
    
    if not parser.load_file():
        return {"file": Path(file_path).name, "status": "❌ Load Failed"}
    
    if not parser.parse_all():
        return {"file": Path(file_path).name, "status": "❌ Parse Failed"}
    
    # Collect summary statistics
    result = {
        "file": Path(file_path).name,
        "status": "✅ Success",
        "version": parser.header.version,
        "vertices": parser.header.vertices.count,
        "bones": parser.header.bones.count,
        "animations": parser.header.animations.count,
        "skins": parser.header.views.count,
        "textures": parser.header.textures.count,
    }
    
    # Check parsing completeness
    parsed_components = 0
    total_components = 0
    
    if parser.header.vertices.count > 0:
        total_components += 1
        if len(parser.vertices) > 0:
            parsed_components += 1
    
    if parser.header.bones.count > 0:
        total_components += 1
        if len(parser.bones) > 0:
            parsed_components += 1
    
    if parser.header.animations.count > 0:
        total_components += 1
        if len(parser.sequences) > 0:
            parsed_components += 1
    
    if parser.header.textures.count > 0:
        total_components += 1
        if len(parser.textures) > 0:
            parsed_components += 1
    
    if parser.header.views.count > 0:
        total_components += 1
        if len(parser.model_views) > 0:
            parsed_components += 1
    
    result["completeness"] = f"{parsed_components}/{total_components}"
    
    return result

@click.command()
@click.argument("m2_files", nargs=-1, type=click.Path(exists=True))
@click.option("--verbose", "-v", is_flag=True, help="Show detailed output")
def main(m2_files, verbose):
    """Batch test M2 models for parser validation"""
    
    if not m2_files:
        # Default to sample data
        sample_dir = Path("~/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/1.12.1/m2").expanduser()
        if sample_dir.exists():
            m2_files = list(sample_dir.glob("*.m2"))
            console.print(f"[yellow]Using default sample models from {sample_dir}[/yellow]\n")
        else:
            console.print("[red]No M2 files specified and sample directory not found[/red]")
            return
    
    # Create results table
    table = Table(title="M2 Parser Batch Test Results")
    table.add_column("Model", style="cyan", no_wrap=True)
    table.add_column("Status", style="green")
    table.add_column("Version", style="yellow", justify="center")
    table.add_column("Vertices", style="blue", justify="right")
    table.add_column("Bones", style="magenta", justify="right")
    table.add_column("Anims", style="red", justify="right")
    table.add_column("Skins", style="cyan", justify="right")
    table.add_column("Textures", style="yellow", justify="right")
    table.add_column("Complete", style="green", justify="center")
    
    # Test each model
    for m2_file in m2_files:
        if verbose:
            console.print(f"Testing {Path(m2_file).name}...")
        
        result = test_model(str(m2_file))
        
        if result["status"] == "✅ Success":
            table.add_row(
                result["file"],
                result["status"],
                str(result.get("version", "-")),
                str(result.get("vertices", "-")),
                str(result.get("bones", "-")),
                str(result.get("animations", "-")),
                str(result.get("skins", "-")),
                str(result.get("textures", "-")),
                result.get("completeness", "-")
            )
        else:
            table.add_row(
                result["file"],
                result["status"],
                "-", "-", "-", "-", "-", "-", "-"
            )
    
    console.print(table)
    
    # Summary statistics
    successful = sum(1 for m2 in m2_files if test_model(str(m2))["status"] == "✅ Success")
    total = len(m2_files)
    
    console.print(f"\n[bold]Summary:[/bold] {successful}/{total} models parsed successfully")
    
    if successful == total:
        console.print("[bold green]✅ All models parsed successfully![/bold green]")
    elif successful > 0:
        console.print(f"[yellow]⚠️  {total - successful} model(s) failed to parse[/yellow]")
    else:
        console.print("[red]❌ No models parsed successfully[/red]")

if __name__ == "__main__":
    main()