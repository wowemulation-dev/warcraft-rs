#!/usr/bin/env python3
"""
Enhanced M2 Parser with complete Rust parity
Extends the base parser with the missing critical features.
"""

import struct
import math
import click
from pathlib import Path
from rich.console import Console
from rich.table import Table
from rich.tree import Tree
from rich.panel import Panel
from dataclasses import dataclass
from typing import List, Optional, Tuple, Dict, Any

# Import base parser
from parser import ComprehensiveM2Parser, Vec3, Vec2, M2Vertex, M2Bone, console

# Import our new modules
from quaternion import M2CompQuat, M2Track, M2TrackBase
from coordinate_systems import CoordinateSystem, CoordinateTransformer, transform_position, transform_quaternion, Vec3 as CoordVec3, Quaternion as CoordQuat
from validation import ValidationMode, M2VertexValidator, create_validator


@dataclass
class EnhancedM2Bone(M2Bone):
    """Enhanced M2 Bone with animation track support"""
    # Animation tracks (parsed when available)
    translation_track: Optional[M2Track] = None
    rotation_track: Optional[M2Track] = None  # Contains M2CompQuat values
    scale_track: Optional[M2Track] = None
    # Parsed animation data
    rotations: List[M2CompQuat] = None
    translations: List[Vec3] = None
    scales: List[Vec3] = None


class EnhancedM2Parser(ComprehensiveM2Parser):
    """Enhanced M2 parser with advanced features matching Rust implementation"""
    
    def __init__(self, file_path: str, validation_mode: str = "permissive", coordinate_system: Optional[str] = None):
        super().__init__(file_path)
        
        # Enhanced features
        self.validator = create_validator(validation_mode)
        self.coordinate_transformer = None
        if coordinate_system:
            coord_sys = CoordinateSystem(coordinate_system.lower())
            self.coordinate_transformer = CoordinateTransformer(coord_sys)
        
        # Enhanced data storage
        self.enhanced_bones: List[EnhancedM2Bone] = []
        self.validation_stats: Dict[str, int] = {}
        self.animation_data: Dict[str, Any] = {}
    
    def parse_vertices(self) -> bool:
        """Parse vertex data with enhanced validation and coordinate transformation"""
        try:
            if not self.header or self.header.vertices.count == 0:
                return False
            
            self.vertices = []
            vertex_size = 48
            bone_count = self.header.bones.count if self.header else None
            
            raw_vertices = []
            
            for i in range(min(self.header.vertices.count, 1000)):
                offset = self.header.vertices.offset + (i * vertex_size)
                if offset + vertex_size > len(self.data):
                    break
                
                # Parse vertex structure
                pos = self.read_vec3(offset)
                bone_weights = list(struct.unpack('4B', self.data[offset+12:offset+16]))
                bone_indices = list(struct.unpack('4B', self.data[offset+16:offset+20]))
                normal = self.read_vec3(offset + 20)
                tex_coords = self.read_vec2(offset + 32)
                tex_coords2 = self.read_vec2(offset + 40)
                
                # Create vertex data for validation
                vertex_data = {
                    'position': [pos.x, pos.y, pos.z],
                    'bone_weights': bone_weights,
                    'bone_indices': bone_indices,
                    'normal': [normal.x, normal.y, normal.z],
                    'tex_coords': [tex_coords.x, tex_coords.y],
                    'tex_coords2': [tex_coords2.x, tex_coords2.y],
                }
                
                # Apply validation
                validated_data = self.validator.validate_vertex(vertex_data, bone_count)
                
                # Apply coordinate transformation if specified
                if self.coordinate_transformer:
                    pos_vec = CoordVec3(*validated_data['position'])
                    transformed_pos = self.coordinate_transformer.transform_position(pos_vec)
                    validated_data['position'] = [transformed_pos.x, transformed_pos.y, transformed_pos.z]
                    
                    norm_vec = CoordVec3(*validated_data['normal'])
                    transformed_norm = self.coordinate_transformer.transform_position(norm_vec)
                    validated_data['normal'] = [transformed_norm.x, transformed_norm.y, transformed_norm.z]
                
                # Create final vertex
                vertex = M2Vertex(
                    position=Vec3(*validated_data['position']),
                    bone_weights=validated_data['bone_weights'],
                    bone_indices=validated_data['bone_indices'],
                    normal=Vec3(*validated_data['normal']),
                    tex_coords=Vec2(*validated_data['tex_coords']),
                    tex_coords2=Vec2(*validated_data['tex_coords2'])
                )
                self.vertices.append(vertex)
                raw_vertices.append(validated_data)
            
            # Calculate validation statistics
            self.validation_stats = self.validator.get_validation_stats(raw_vertices)
            
            return True
            
        except Exception as e:
            console.print(f"[red]Enhanced vertex parsing failed: {e}[/red]")
            return False
    
    def parse_bones_enhanced(self) -> bool:
        """Parse bone data with animation track support"""
        try:
            if not self.header or self.header.bones.count == 0:
                return False
            
            # First parse bones normally
            if not super().parse_bones():
                return False
            
            # Now enhance with animation data
            self.enhanced_bones = []
            version = self.header.version if self.header else 256
            
            for i, base_bone in enumerate(self.bones[:min(len(self.bones), 100)]):
                if i >= self.header.bones.count:
                    break
                
                bone_offset = self.header.bones.offset + (i * 112)
                
                # Parse animation tracks
                translation_track = None
                rotation_track = None
                scale_track = None
                
                try:
                    track_offset = bone_offset + 12
                    
                    # Translation track
                    translation_track = M2Track.parse(self.data, track_offset, version)
                    track_offset += 20 if version < 264 else 12
                    
                    # Rotation track (contains M2CompQuat values)
                    rotation_track = M2Track.parse(self.data, track_offset, version)
                    track_offset += 20 if version < 264 else 12
                    
                    # Scale track
                    scale_track = M2Track.parse(self.data, track_offset, version)
                    
                except Exception as track_error:
                    console.print(f"[yellow]Warning: Failed to parse animation tracks for bone {i}: {track_error}[/yellow]")
                
                # Parse actual animation data
                rotations = []
                translations = []
                scales = []
                
                if rotation_track and rotation_track.has_data():
                    try:
                        rotations = self.parse_rotation_track_data(rotation_track)
                    except Exception as e:
                        console.print(f"[yellow]Warning: Failed to parse rotation data for bone {i}: {e}[/yellow]")
                
                if translation_track and translation_track.has_data():
                    try:
                        translations = self.parse_translation_track_data(translation_track)
                    except Exception as e:
                        console.print(f"[yellow]Warning: Failed to parse translation data for bone {i}: {e}[/yellow]")
                
                if scale_track and scale_track.has_data():
                    try:
                        scales = self.parse_scale_track_data(scale_track)
                    except Exception as e:
                        console.print(f"[yellow]Warning: Failed to parse scale data for bone {i}: {e}[/yellow]")
                
                # Apply coordinate transformation to pivot if specified
                pivot = base_bone.pivot
                if self.coordinate_transformer:
                    pivot_vec = CoordVec3(pivot.x, pivot.y, pivot.z)
                    transformed_pivot = self.coordinate_transformer.transform_position(pivot_vec)
                    pivot = Vec3(transformed_pivot.x, transformed_pivot.y, transformed_pivot.z)
                
                # Create enhanced bone
                enhanced_bone = EnhancedM2Bone(
                    key_bone_id=base_bone.key_bone_id,
                    flags=base_bone.flags,
                    parent=base_bone.parent,
                    submesh_id=base_bone.submesh_id,
                    pivot=pivot,
                    translation_track=translation_track,
                    rotation_track=rotation_track,
                    scale_track=scale_track,
                    rotations=rotations,
                    translations=translations,
                    scales=scales
                )
                self.enhanced_bones.append(enhanced_bone)
            
            return True
            
        except Exception as e:
            console.print(f"[red]Enhanced bone parsing failed: {e}[/red]")
            return False
    
    def parse_rotation_track_data(self, track: M2Track) -> List[M2CompQuat]:
        """Parse rotation track data containing M2CompQuat values"""
        rotations = []
        
        if track.values['count'] == 0:
            return rotations
        
        for i in range(min(track.values['count'], 100)):  # Limit for memory
            offset = track.values['offset'] + (i * 8)
            if offset + 8 <= len(self.data):
                quat = M2CompQuat.parse(self.data, offset)
                
                # Apply coordinate transformation if specified
                if self.coordinate_transformer:
                    x, y, z, w = quat.to_float_quaternion()
                    coord_quat = CoordQuat(x, y, z, w)
                    transformed = self.coordinate_transformer.transform_quaternion(coord_quat)
                    quat = M2CompQuat.from_float_quaternion(transformed.x, transformed.y, transformed.z, transformed.w)
                
                rotations.append(quat)
        
        return rotations
    
    def parse_translation_track_data(self, track: M2Track) -> List[Vec3]:
        """Parse translation track data containing Vec3 values"""
        translations = []
        
        if track.values['count'] == 0:
            return translations
        
        for i in range(min(track.values['count'], 100)):  # Limit for memory
            offset = track.values['offset'] + (i * 12)
            if offset + 12 <= len(self.data):
                translation = self.read_vec3(offset)
                
                # Apply coordinate transformation if specified
                if self.coordinate_transformer:
                    coord_vec = CoordVec3(translation.x, translation.y, translation.z)
                    transformed = self.coordinate_transformer.transform_position(coord_vec)
                    translation = Vec3(transformed.x, transformed.y, transformed.z)
                
                translations.append(translation)
        
        return translations
    
    def parse_scale_track_data(self, track: M2Track) -> List[Vec3]:
        """Parse scale track data containing Vec3 values"""
        scales = []
        
        if track.values['count'] == 0:
            return scales
        
        for i in range(min(track.values['count'], 100)):  # Limit for memory
            offset = track.values['offset'] + (i * 12)
            if offset + 12 <= len(self.data):
                scale = self.read_vec3(offset)
                scales.append(scale)
        
        return scales
    
    def parse_all_enhanced(self) -> bool:
        """Parse all M2 data with enhanced features"""
        console.print("[yellow]Parsing M2 file with enhanced features...[/yellow]")
        
        if not self.parse_header():
            return False
        console.print("[green]✅ Header parsed[/green]")
        
        # Parse vertices with validation
        if self.parse_vertices():
            console.print(f"[green]✅ Parsed {len(self.vertices)} vertices with validation[/green]")
        
        # Parse bones with animation tracks
        if self.parse_bones_enhanced():
            console.print(f"[green]✅ Parsed {len(self.enhanced_bones)} bones with animation data[/green]")
        
        # Parse other components normally
        if self.parse_textures():
            console.print(f"[green]✅ Parsed {len(self.textures)} textures[/green]")
        
        if self.parse_sequences():
            console.print(f"[green]✅ Parsed {len(self.sequences)} animation sequences[/green]")
        
        if self.parse_model_views():
            console.print(f"[green]✅ Parsed {len(self.model_views)} skin profiles[/green]")
        
        return True
    
    def create_enhanced_visual_representation(self):
        """Create enhanced visual representation with new features"""
        # Call base implementation first
        super().create_visual_representation()
        
        # Add validation statistics
        if self.validation_stats:
            validation_table = Table(title="🔍 Validation Statistics (New Feature)")
            validation_table.add_column("Metric", style="cyan")
            validation_table.add_column("Count", style="green")
            validation_table.add_column("Percentage", style="yellow")
            
            total = self.validation_stats.get('total_vertices', 1)
            for key, value in self.validation_stats.items():
                if key != 'total_vertices':
                    percentage = f"{(value/total)*100:.1f}%" if total > 0 else "0%"
                    validation_table.add_row(key.replace('_', ' ').title(), str(value), percentage)
            
            console.print(validation_table)
        
        # Enhanced bone animation data visualization
        if self.enhanced_bones:
            anim_tree = Tree("[bold cyan]🎬 Animation Data (New Feature)[/bold cyan]")
            
            bones_with_animation = [b for b in self.enhanced_bones if b.rotations or b.translations or b.scales]
            
            for i, bone in enumerate(bones_with_animation[:10]):  # Show first 10 animated bones
                bone_node = anim_tree.add(f"Bone {bone.key_bone_id}")
                
                if bone.rotations:
                    rot_node = bone_node.add(f"Rotations: {len(bone.rotations)} keyframes")
                    if bone.rotations:
                        first_rot = bone.rotations[0]
                        x, y, z, w = first_rot.to_float_quaternion()
                        rot_node.add(f"First: ({x:.3f}, {y:.3f}, {z:.3f}, {w:.3f})")
                        
                        # Show Euler angles
                        pitch, yaw, roll = first_rot.to_euler_degrees()
                        rot_node.add(f"Euler: Pitch={pitch:.1f}°, Yaw={yaw:.1f}°, Roll={roll:.1f}°")
                
                if bone.translations:
                    trans_node = bone_node.add(f"Translations: {len(bone.translations)} keyframes")
                    if bone.translations:
                        first_trans = bone.translations[0]
                        trans_node.add(f"First: {first_trans}")
                
                if bone.scales:
                    scale_node = bone_node.add(f"Scales: {len(bone.scales)} keyframes")
                    if bone.scales:
                        first_scale = bone.scales[0]
                        scale_node.add(f"First: {first_scale}")
            
            console.print(Panel(anim_tree, title="[bold]Enhanced Animation Data[/bold]"))
        
        # Coordinate transformation info
        if self.coordinate_transformer:
            coord_table = Table(title="🌐 Coordinate Transformation (New Feature)")
            coord_table.add_column("Property", style="cyan")
            coord_table.add_column("Value", style="green")
            
            coord_table.add_row("Target System", str(self.coordinate_transformer.target.value))
            coord_table.add_row("Vertices Transformed", str(len(self.vertices)))
            coord_table.add_row("Bones Transformed", str(len(self.enhanced_bones)))
            
            console.print(coord_table)


@click.command()
@click.argument("m2_file", type=click.Path(exists=True))
@click.option("--export", "-e", type=click.Path(), help="Export parsed data to JSON")
@click.option("--validation-mode", "-v", default="permissive", 
              type=click.Choice(["strict", "permissive", "none"]), 
              help="Validation mode: strict (fix all issues), permissive (fix corruption), none (no fixes)")
@click.option("--coordinate-system", "-c", 
              type=click.Choice(["blender", "unity", "unreal"]), 
              help="Transform coordinates to target system")
@click.option("--show-quaternions", "-q", is_flag=True, 
              help="Show detailed quaternion animation data")
def main(m2_file: str, export: Optional[str], validation_mode: str, 
         coordinate_system: Optional[str], show_quaternions: bool):
    """Enhanced M2 Parser - Parse WoW M2 models with Rust parity features
    
    NEW FEATURES:
    - 🔧 Quaternion support with X-component negation (matches pywowlib)
    - 🌐 Coordinate transformations for Blender/Unity/Unreal
    - 🛡️  Enhanced validation modes (Strict/Permissive/None) 
    - 🎬 Complete animation track parsing including rotations
    """
    
    console.print(f"\n[bold magenta]═══ Enhanced M2 Parser (Rust Parity) ═══[/bold magenta]")
    console.print(f"[cyan]File:[/cyan] {m2_file}")
    console.print(f"[cyan]Size:[/cyan] {Path(m2_file).stat().st_size:,} bytes")
    console.print(f"[cyan]Validation Mode:[/cyan] {validation_mode}")
    if coordinate_system:
        console.print(f"[cyan]Coordinate System:[/cyan] {coordinate_system}")
    console.print()
    
    parser = EnhancedM2Parser(m2_file, validation_mode, coordinate_system)
    
    # Load and parse with enhanced features
    if not parser.load_file():
        console.print("[red]Failed to load file[/red]")
        return
    
    if not parser.parse_all_enhanced():
        console.print("[red]Failed to parse M2 data[/red]")
        return
    
    # Display enhanced visual representation
    parser.create_enhanced_visual_representation()
    
    # Show quaternion details if requested
    if show_quaternions and parser.enhanced_bones:
        console.print("\n[bold cyan]🎯 Quaternion Animation Details[/bold cyan]")
        for i, bone in enumerate(parser.enhanced_bones[:5]):  # First 5 bones
            if bone.rotations:
                console.print(f"\n[yellow]Bone {bone.key_bone_id} Rotations:[/yellow]")
                for j, quat in enumerate(bone.rotations[:3]):  # First 3 keyframes
                    x, y, z, w = quat.to_float_quaternion()
                    pitch, yaw, roll = quat.to_euler_degrees()
                    console.print(f"  Frame {j}: Quat({x:.3f}, {y:.3f}, {z:.3f}, {w:.3f}) = "
                                f"Euler({pitch:.1f}°, {yaw:.1f}°, {roll:.1f}°)")
    
    # Export if requested
    if export:
        import json
        
        export_data = {
            "file": str(m2_file),
            "enhanced_features": {
                "validation_mode": validation_mode,
                "coordinate_system": coordinate_system,
                "validation_stats": parser.validation_stats,
            },
            "header": {
                "magic": parser.header.magic,
                "version": parser.header.version,
                "vertices": parser.header.vertices.count,
                "bones": parser.header.bones.count,
                "animations": parser.header.animations.count,
            },
            "animation_data": {
                "bones_with_rotations": len([b for b in parser.enhanced_bones if b.rotations]),
                "bones_with_translations": len([b for b in parser.enhanced_bones if b.translations]),
                "bones_with_scales": len([b for b in parser.enhanced_bones if b.scales]),
                "total_rotation_keyframes": sum(len(b.rotations) for b in parser.enhanced_bones if b.rotations),
            }
        }
        
        # Add sample data
        if parser.enhanced_bones:
            sample_bone = parser.enhanced_bones[0]
            export_data["sample_bone"] = {
                "key_bone_id": sample_bone.key_bone_id,
                "pivot": [sample_bone.pivot.x, sample_bone.pivot.y, sample_bone.pivot.z],
                "has_rotations": len(sample_bone.rotations) if sample_bone.rotations else 0,
                "has_translations": len(sample_bone.translations) if sample_bone.translations else 0,
            }
            
            if sample_bone.rotations:
                quat = sample_bone.rotations[0]
                x, y, z, w = quat.to_float_quaternion()
                export_data["sample_bone"]["first_rotation"] = {
                    "quaternion": [x, y, z, w],
                    "euler_degrees": list(quat.to_euler_degrees())
                }
        
        with open(export, 'w') as f:
            json.dump(export_data, f, indent=2)
        
        console.print(f"\n[green]✅ Enhanced data exported to {export}[/green]")
    
    console.print("\n[bold green]═══ Enhanced Parsing Complete ═══[/bold green]")
    console.print("[dim]Features implemented: Quaternions ✅ Coordinates ✅ Validation ✅ Animation Tracks ✅[/dim]\n")


if __name__ == "__main__":
    main()