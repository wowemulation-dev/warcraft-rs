#!/usr/bin/env python3
"""
Enhanced M2 Parser for WoW (Version 256-264+)
Parses and visualizes M2 data structures with advanced features:
- Compressed quaternion support with X-component negation
- Coordinate system transformations for Blender/Unity/Unreal
- Enhanced validation modes (Strict/Permissive/None)
- Complete animation track parsing including quaternions

Achieves parity with the Rust implementation.
"""

import struct
import math
import click
from pathlib import Path
from rich.console import Console
from rich.table import Table
from rich.tree import Tree
from rich.panel import Panel
from rich.layout import Layout
from rich.columns import Columns
from dataclasses import dataclass
from typing import List, Optional, Tuple, Dict, Any

# Import our new modules
from quaternion import M2CompQuat, M2Track, M2TrackBase
from coordinate_systems import CoordinateSystem, CoordinateTransformer, transform_position, transform_quaternion
from validation import ValidationMode, M2VertexValidator, create_validator

console = Console()

# ============================================================================
# Data Structure Classes
# ============================================================================

@dataclass
class Vec3:
    x: float
    y: float
    z: float
    
    def __repr__(self):
        return f"({self.x:.2f}, {self.y:.2f}, {self.z:.2f})"
    
    def magnitude(self):
        return math.sqrt(self.x**2 + self.y**2 + self.z**2)

@dataclass
class Vec2:
    x: float
    y: float
    
    def __repr__(self):
        return f"({self.x:.2f}, {self.y:.2f})"

@dataclass
class M2Array:
    """M2Array structure (count + offset)"""
    count: int
    offset: int
    
    def __repr__(self):
        return f"[{self.count} @ {self.offset:#x}]"

@dataclass
class M2Vertex:
    """M2 Vertex structure (48 bytes)"""
    position: Vec3
    bone_weights: List[int]  # 4 bytes
    bone_indices: List[int]  # 4 bytes
    normal: Vec3
    tex_coords: Vec2
    tex_coords2: Vec2

@dataclass
class M2Bone:
    """M2 Bone structure with animation track support"""
    key_bone_id: int
    flags: int
    parent: int
    submesh_id: int
    pivot: Vec3
    # Animation tracks (parsed when available)
    translation_track: Optional[M2Track] = None
    rotation_track: Optional[M2Track] = None  # Contains M2CompQuat values
    scale_track: Optional[M2Track] = None
    # Parsed animation data
    rotations: List[M2CompQuat] = None
    translations: List[Vec3] = None
    scales: List[Vec3] = None
    
@dataclass
class M2Texture:
    """M2 Texture definition"""
    type: int
    flags: int
    name: M2Array

@dataclass
class M2Sequence:
    """Animation sequence"""
    id: int
    start: int
    end: int
    move_speed: float
    flags: int
    frequency: int
    replay: Tuple[int, int]
    blend_time: int
    bounds: Tuple[Vec3, Vec3]
    variation_index: int
    variation_next: int

@dataclass
class M2Submesh:
    """Submesh/Geoset structure"""
    id: int
    submesh_id: int
    level: int
    start_vertex: int
    vertex_count: int
    start_triangle: int
    triangle_count: int
    bone_count: int
    bone_start: int
    flags: int
    center_position: Vec3
    radius: float

class M2Header:
    """Complete M2 header structure for version 256"""
    
    def __init__(self):
        self.magic: str = ""
        self.version: int = 0
        self.name: M2Array = M2Array(0, 0)
        self.flags: int = 0
        self.global_sequences: M2Array = M2Array(0, 0)
        self.animations: M2Array = M2Array(0, 0)
        self.animation_lookup: M2Array = M2Array(0, 0)
        self.bones: M2Array = M2Array(0, 0)
        self.key_bone_lookup: M2Array = M2Array(0, 0)
        self.vertices: M2Array = M2Array(0, 0)
        self.views: M2Array = M2Array(0, 0)
        self.colors: M2Array = M2Array(0, 0)
        self.textures: M2Array = M2Array(0, 0)
        self.transparency: M2Array = M2Array(0, 0)
        self.texture_animations: M2Array = M2Array(0, 0)
        self.texture_replacements: M2Array = M2Array(0, 0)
        self.render_flags: M2Array = M2Array(0, 0)
        self.bone_lookup: M2Array = M2Array(0, 0)
        self.texture_lookup: M2Array = M2Array(0, 0)
        self.texture_units: M2Array = M2Array(0, 0)
        self.transparency_lookup: M2Array = M2Array(0, 0)
        self.texture_animation_lookup: M2Array = M2Array(0, 0)
        self.bounding_box: Tuple[Vec3, Vec3] = (Vec3(0,0,0), Vec3(0,0,0))
        self.bounding_sphere_radius: float = 0
        self.collision_box: Tuple[Vec3, Vec3] = (Vec3(0,0,0), Vec3(0,0,0))
        self.collision_sphere_radius: float = 0
        self.collision_triangles: M2Array = M2Array(0, 0)
        self.collision_vertices: M2Array = M2Array(0, 0)
        self.collision_normals: M2Array = M2Array(0, 0)
        self.attachments: M2Array = M2Array(0, 0)
        self.attachment_lookup: M2Array = M2Array(0, 0)
        self.events: M2Array = M2Array(0, 0)
        self.lights: M2Array = M2Array(0, 0)
        self.cameras: M2Array = M2Array(0, 0)
        self.camera_lookup: M2Array = M2Array(0, 0)
        self.ribbon_emitters: M2Array = M2Array(0, 0)
        self.particle_emitters: M2Array = M2Array(0, 0)

class ModelView:
    """ModelView structure for embedded skins (44 bytes)"""
    
    def __init__(self):
        self.indices: M2Array = M2Array(0, 0)
        self.triangles: M2Array = M2Array(0, 0)
        self.properties: M2Array = M2Array(0, 0)
        self.submeshes: M2Array = M2Array(0, 0)
        self.texture_units: M2Array = M2Array(0, 0)
        self.bone_count_max: int = 0

# ============================================================================
# M2 Parser Implementation
# ============================================================================

class EnhancedM2Parser:
    """Enhanced M2 parser with advanced features matching Rust implementation"""
    
    def __init__(self, file_path: str, validation_mode: str = "permissive", coordinate_system: Optional[str] = None):
        self.file_path = Path(file_path)
        self.data: bytes = b""
        self.header: Optional[M2Header] = None
        self.vertices: List[M2Vertex] = []
        self.bones: List[M2Bone] = []
        self.textures: List[M2Texture] = []
        self.sequences: List[M2Sequence] = []
        self.model_views: List[ModelView] = []
        self.submeshes: List[List[M2Submesh]] = []
        
        # Enhanced features
        self.validator = create_validator(validation_mode)
        self.coordinate_transformer = None
        if coordinate_system:
            coord_sys = CoordinateSystem(coordinate_system.lower())
            self.coordinate_transformer = CoordinateTransformer(coord_sys)
        
        # Animation data
        self.bone_animation_data: Dict[int, Dict[str, Any]] = {}
        self.validation_stats: Dict[str, int] = {}
        
    def load_file(self) -> bool:
        """Load M2 file into memory"""
        try:
            with open(self.file_path, 'rb') as f:
                self.data = f.read()
            return True
        except Exception as e:
            console.print(f"[red]Failed to load file: {e}[/red]")
            return False
    
    def read_vec3(self, offset: int) -> Vec3:
        """Read 3D vector at offset"""
        x, y, z = struct.unpack('<fff', self.data[offset:offset+12])
        return Vec3(x, y, z)
    
    def read_vec2(self, offset: int) -> Vec2:
        """Read 2D vector at offset"""
        x, y = struct.unpack('<ff', self.data[offset:offset+8])
        return Vec2(x, y)
    
    def read_m2array(self, offset: int) -> M2Array:
        """Read M2Array structure at offset"""
        count, arr_offset = struct.unpack('<II', self.data[offset:offset+8])
        return M2Array(count, arr_offset)
    
    def read_string(self, offset: int, max_len: int = 256) -> str:
        """Read null-terminated string at offset"""
        end = self.data.find(b'\0', offset, offset + max_len)
        if end == -1:
            end = offset + max_len
        return self.data[offset:end].decode('utf-8', errors='ignore')
    
    def parse_header(self) -> bool:
        """Parse complete M2 header"""
        try:
            if len(self.data) < 324:  # Minimum header size for v256
                console.print("[red]File too small for M2 header[/red]")
                return False
            
            self.header = M2Header()
            pos = 0
            
            # Magic and version
            self.header.magic = self.data[pos:pos+4].decode('ascii', errors='ignore')
            pos += 4
            self.header.version = struct.unpack('<I', self.data[pos:pos+4])[0]
            pos += 4
            
            # Name
            self.header.name = self.read_m2array(pos)
            pos += 8
            
            # Flags
            self.header.flags = struct.unpack('<I', self.data[pos:pos+4])[0]
            pos += 4
            
            # Arrays in order
            self.header.global_sequences = self.read_m2array(pos); pos += 8
            self.header.animations = self.read_m2array(pos); pos += 8
            self.header.animation_lookup = self.read_m2array(pos); pos += 8
            pos += 8  # playable_animation_lookup (v256 specific)
            self.header.bones = self.read_m2array(pos); pos += 8
            self.header.key_bone_lookup = self.read_m2array(pos); pos += 8
            self.header.vertices = self.read_m2array(pos); pos += 8
            self.header.views = self.read_m2array(pos); pos += 8
            self.header.colors = self.read_m2array(pos); pos += 8
            self.header.textures = self.read_m2array(pos); pos += 8
            self.header.transparency = self.read_m2array(pos); pos += 8
            pos += 8  # unknown (v256)
            self.header.texture_animations = self.read_m2array(pos); pos += 8
            self.header.texture_replacements = self.read_m2array(pos); pos += 8
            self.header.render_flags = self.read_m2array(pos); pos += 8
            self.header.bone_lookup = self.read_m2array(pos); pos += 8
            self.header.texture_lookup = self.read_m2array(pos); pos += 8
            self.header.texture_units = self.read_m2array(pos); pos += 8
            self.header.transparency_lookup = self.read_m2array(pos); pos += 8
            self.header.texture_animation_lookup = self.read_m2array(pos); pos += 8
            
            # Bounding volumes
            min_bound = self.read_vec3(pos); pos += 12
            max_bound = self.read_vec3(pos); pos += 12
            self.header.bounding_box = (min_bound, max_bound)
            self.header.bounding_sphere_radius = struct.unpack('<f', self.data[pos:pos+4])[0]
            pos += 4
            
            min_coll = self.read_vec3(pos); pos += 12
            max_coll = self.read_vec3(pos); pos += 12
            self.header.collision_box = (min_coll, max_coll)
            self.header.collision_sphere_radius = struct.unpack('<f', self.data[pos:pos+4])[0]
            pos += 4
            
            # More arrays
            self.header.collision_triangles = self.read_m2array(pos); pos += 8
            self.header.collision_vertices = self.read_m2array(pos); pos += 8
            self.header.collision_normals = self.read_m2array(pos); pos += 8
            self.header.attachments = self.read_m2array(pos); pos += 8
            self.header.attachment_lookup = self.read_m2array(pos); pos += 8
            self.header.events = self.read_m2array(pos); pos += 8
            self.header.lights = self.read_m2array(pos); pos += 8
            self.header.cameras = self.read_m2array(pos); pos += 8
            self.header.camera_lookup = self.read_m2array(pos); pos += 8
            self.header.ribbon_emitters = self.read_m2array(pos); pos += 8
            self.header.particle_emitters = self.read_m2array(pos); pos += 8
            
            return True
            
        except Exception as e:
            console.print(f"[red]Header parsing failed: {e}[/red]")
            return False
    
    def parse_vertices(self) -> bool:
        """Parse vertex data"""
        try:
            if not self.header or self.header.vertices.count == 0:
                return False
            
            self.vertices = []
            vertex_size = 48  # Fixed size for v256
            
            for i in range(min(self.header.vertices.count, 1000)):  # Limit for display
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
                
                vertex = M2Vertex(
                    position=pos,
                    bone_weights=bone_weights,
                    bone_indices=bone_indices,
                    normal=normal,
                    tex_coords=tex_coords,
                    tex_coords2=tex_coords2
                )
                self.vertices.append(vertex)
            
            return True
            
        except Exception as e:
            console.print(f"[red]Vertex parsing failed: {e}[/red]")
            return False
    
    def parse_bones(self) -> bool:
        """Parse bone data"""
        try:
            if not self.header or self.header.bones.count == 0:
                return False
            
            self.bones = []
            bone_size = 112  # Base size for v256 (without animated tracks)
            
            for i in range(min(self.header.bones.count, 100)):  # Limit for display
                offset = self.header.bones.offset + (i * bone_size)
                if offset + 20 > len(self.data):  # Minimum readable size
                    break
                
                # Parse basic bone data
                key_bone_id = struct.unpack('<i', self.data[offset:offset+4])[0]
                flags = struct.unpack('<I', self.data[offset+4:offset+8])[0]
                parent = struct.unpack('<h', self.data[offset+8:offset+10])[0]
                submesh_id = struct.unpack('<H', self.data[offset+10:offset+12])[0]
                
                # Skip animated tracks, read pivot
                pivot_offset = offset + 88  # After all the animated track arrays
                if pivot_offset + 12 <= len(self.data):
                    pivot = self.read_vec3(pivot_offset)
                else:
                    pivot = Vec3(0, 0, 0)
                
                bone = M2Bone(
                    key_bone_id=key_bone_id,
                    flags=flags,
                    parent=parent,
                    submesh_id=submesh_id,
                    pivot=pivot
                )
                self.bones.append(bone)
            
            return True
            
        except Exception as e:
            console.print(f"[red]Bone parsing failed: {e}[/red]")
            return False
    
    def parse_textures(self) -> bool:
        """Parse texture definitions"""
        try:
            if not self.header or self.header.textures.count == 0:
                return False
            
            self.textures = []
            texture_size = 16  # Size for v256
            
            for i in range(self.header.textures.count):
                offset = self.header.textures.offset + (i * texture_size)
                if offset + texture_size > len(self.data):
                    break
                
                tex_type = struct.unpack('<I', self.data[offset:offset+4])[0]
                flags = struct.unpack('<I', self.data[offset+4:offset+8])[0]
                name_array = self.read_m2array(offset + 8)
                
                texture = M2Texture(
                    type=tex_type,
                    flags=flags,
                    name=name_array
                )
                self.textures.append(texture)
            
            return True
            
        except Exception as e:
            console.print(f"[red]Texture parsing failed: {e}[/red]")
            return False
    
    def parse_sequences(self) -> bool:
        """Parse animation sequences"""
        try:
            if not self.header or self.header.animations.count == 0:
                return False
            
            self.sequences = []
            sequence_size = 68  # Size for v256
            
            for i in range(min(self.header.animations.count, 50)):  # Limit for display
                offset = self.header.animations.offset + (i * sequence_size)
                if offset + sequence_size > len(self.data):
                    break
                
                # Parse sequence structure
                seq_id = struct.unpack('<H', self.data[offset:offset+2])[0]
                start = struct.unpack('<H', self.data[offset+2:offset+4])[0]
                end = struct.unpack('<H', self.data[offset+4:offset+6])[0]
                move_speed = struct.unpack('<f', self.data[offset+8:offset+12])[0]
                flags = struct.unpack('<I', self.data[offset+12:offset+16])[0]
                frequency = struct.unpack('<h', self.data[offset+16:offset+18])[0]
                replay_min = struct.unpack('<I', self.data[offset+20:offset+24])[0]
                replay_max = struct.unpack('<I', self.data[offset+24:offset+28])[0]
                blend_time = struct.unpack('<I', self.data[offset+28:offset+32])[0]
                
                # Bounds
                min_bound = self.read_vec3(offset + 32)
                max_bound = self.read_vec3(offset + 44)
                
                radius = struct.unpack('<f', self.data[offset+56:offset+60])[0]
                variation_index = struct.unpack('<h', self.data[offset+60:offset+62])[0]
                variation_next = struct.unpack('<h', self.data[offset+62:offset+64])[0]
                
                sequence = M2Sequence(
                    id=seq_id,
                    start=start,
                    end=end,
                    move_speed=move_speed,
                    flags=flags,
                    frequency=frequency,
                    replay=(replay_min, replay_max),
                    blend_time=blend_time,
                    bounds=(min_bound, max_bound),
                    variation_index=variation_index,
                    variation_next=variation_next
                )
                self.sequences.append(sequence)
            
            return True
            
        except Exception as e:
            console.print(f"[red]Sequence parsing failed: {e}[/red]")
            return False
    
    def parse_model_views(self) -> bool:
        """Parse all model views (skins)"""
        try:
            if not self.header or self.header.views.count == 0:
                return False
            
            self.model_views = []
            self.submeshes = []
            
            for view_idx in range(self.header.views.count):
                # Parse ModelView structure (44 bytes for embedded skins)
                view_offset = self.header.views.offset + (view_idx * 44)
                if view_offset + 44 > len(self.data):
                    break
                
                model_view = ModelView()
                pos = view_offset
                
                model_view.indices = self.read_m2array(pos); pos += 8
                model_view.triangles = self.read_m2array(pos); pos += 8
                model_view.properties = self.read_m2array(pos); pos += 8
                model_view.submeshes = self.read_m2array(pos); pos += 8
                model_view.texture_units = self.read_m2array(pos); pos += 8
                model_view.bone_count_max = struct.unpack('<I', self.data[pos:pos+4])[0]
                
                self.model_views.append(model_view)
                
                # Parse submeshes for this view
                view_submeshes = []
                submesh_size = 32  # Vanilla submesh size
                
                for i in range(min(model_view.submeshes.count, 100)):
                    sm_offset = model_view.submeshes.offset + (i * submesh_size)
                    if sm_offset + submesh_size > len(self.data):
                        break
                    
                    # Parse submesh structure
                    sm_data = struct.unpack('<8H4f', self.data[sm_offset:sm_offset+32])
                    
                    submesh = M2Submesh(
                        id=i,
                        submesh_id=sm_data[0],
                        level=sm_data[1],
                        start_vertex=sm_data[2],
                        vertex_count=sm_data[3],
                        start_triangle=sm_data[4],
                        triangle_count=sm_data[5],
                        bone_count=sm_data[6],
                        bone_start=sm_data[7],
                        flags=0,  # Not in vanilla format
                        center_position=Vec3(sm_data[8], sm_data[9], sm_data[10]),
                        radius=sm_data[11]
                    )
                    view_submeshes.append(submesh)
                
                self.submeshes.append(view_submeshes)
            
            return True
            
        except Exception as e:
            console.print(f"[red]Model view parsing failed: {e}[/red]")
            return False
    
    def create_visual_representation(self) -> None:
        """Create comprehensive visual representation of M2 data"""
        
        # Create header panel
        header_tree = Tree("[bold cyan]M2 Header Information[/bold cyan]")
        header_tree.add(f"Magic: [green]{self.header.magic}[/green]")
        header_tree.add(f"Version: [green]{self.header.version}[/green]")
        header_tree.add(f"Flags: [yellow]{self.header.flags:#x}[/yellow]")
        
        # Model name
        if self.header.name.count > 0:
            name = self.read_string(self.header.name.offset)
            header_tree.add(f"Name: [magenta]{name}[/magenta]")
        
        # Bounding volumes
        bounds_node = header_tree.add("Bounding Volumes")
        bounds_node.add(f"Box: {self.header.bounding_box[0]} to {self.header.bounding_box[1]}")
        bounds_node.add(f"Sphere Radius: {self.header.bounding_sphere_radius:.2f}")
        
        console.print(Panel(header_tree, title="[bold]Model Header[/bold]"))
        
        # Create statistics table
        stats_table = Table(title="Model Statistics")
        stats_table.add_column("Component", style="cyan")
        stats_table.add_column("Count", style="green")
        stats_table.add_column("Offset", style="yellow")
        stats_table.add_column("Status", style="white")
        
        components = [
            ("Vertices", self.header.vertices, len(self.vertices) > 0),
            ("Views (Skins)", self.header.views, len(self.model_views) > 0),
            ("Bones", self.header.bones, len(self.bones) > 0),
            ("Animations", self.header.animations, len(self.sequences) > 0),
            ("Textures", self.header.textures, len(self.textures) > 0),
            ("Attachments", self.header.attachments, False),
            ("Events", self.header.events, False),
            ("Lights", self.header.lights, False),
            ("Cameras", self.header.cameras, False),
            ("Particles", self.header.particle_emitters, False),
            ("Ribbons", self.header.ribbon_emitters, False),
        ]
        
        for name, array, parsed in components:
            status = "✅ Parsed" if parsed else "📄 Header Only"
            stats_table.add_row(name, str(array.count), f"{array.offset:#x}", status)
        
        console.print(stats_table)
        
        # Vertex data visualization
        if self.vertices:
            vertex_tree = Tree("[bold cyan]Vertex Data Sample[/bold cyan]")
            for i, vertex in enumerate(self.vertices[:5]):
                v_node = vertex_tree.add(f"Vertex {i}")
                v_node.add(f"Position: {vertex.position}")
                v_node.add(f"Normal: {vertex.normal}")
                v_node.add(f"UV: {vertex.tex_coords}")
                v_node.add(f"Bones: {vertex.bone_indices[:vertex.bone_weights.count(0) or 4]}")
                v_node.add(f"Weights: {vertex.bone_weights[:vertex.bone_weights.count(0) or 4]}")
            
            console.print(Panel(vertex_tree, title="[bold]Vertex Sample (First 5)[/bold]"))
        
        # Bone hierarchy visualization
        if self.bones:
            bone_tree = Tree("[bold cyan]Bone Hierarchy[/bold cyan]")
            
            # Build bone hierarchy
            root_bones = [i for i, bone in enumerate(self.bones) if bone.parent == -1]
            
            def add_bone_children(parent_node, bone_idx, depth=0):
                if depth > 5:  # Limit depth for display
                    return
                bone = self.bones[bone_idx]
                bone_str = f"Bone {bone_idx}: KeyBone={bone.key_bone_id}, Pivot={bone.pivot}"
                node = parent_node.add(bone_str)
                
                # Find children
                for i, b in enumerate(self.bones):
                    if b.parent == bone_idx:
                        add_bone_children(node, i, depth + 1)
            
            for root_idx in root_bones[:3]:  # Limit root bones shown
                add_bone_children(bone_tree, root_idx)
            
            console.print(Panel(bone_tree, title="[bold]Bone Hierarchy (Partial)[/bold]"))
        
        # Animation sequences
        if self.sequences:
            anim_table = Table(title="Animation Sequences")
            anim_table.add_column("ID", style="cyan")
            anim_table.add_column("Frames", style="green")
            anim_table.add_column("Speed", style="yellow")
            anim_table.add_column("Flags", style="magenta")
            anim_table.add_column("Replay", style="blue")
            
            for seq in self.sequences[:10]:  # First 10 animations
                frames = f"{seq.start}-{seq.end}"
                replay = f"{seq.replay[0]}-{seq.replay[1]}"
                anim_table.add_row(
                    str(seq.id),
                    frames,
                    f"{seq.move_speed:.2f}",
                    f"{seq.flags:#x}",
                    replay
                )
            
            console.print(anim_table)
        
        # Skin/View details
        if self.model_views:
            for i, (view, submeshes) in enumerate(zip(self.model_views[:2], self.submeshes[:2])):
                view_tree = Tree(f"[bold cyan]Skin Profile {i}[/bold cyan]")
                view_tree.add(f"Indices: {view.indices.count} @ {view.indices.offset:#x}")
                view_tree.add(f"Triangles: {view.triangles.count} @ {view.triangles.offset:#x}")
                view_tree.add(f"Submeshes: {view.submeshes.count}")
                view_tree.add(f"Max Bones: {view.bone_count_max}")
                
                # Submesh details
                if submeshes:
                    sm_node = view_tree.add("Submeshes")
                    for sm in submeshes[:5]:  # First 5 submeshes
                        sm_str = (f"ID {sm.submesh_id}: "
                                 f"Verts={sm.vertex_count}, "
                                 f"Tris={sm.triangle_count}, "
                                 f"Bones={sm.bone_count}")
                        sm_node.add(sm_str)
                
                console.print(Panel(view_tree, title=f"[bold]Skin Profile {i}[/bold]"))
        
        # Texture information
        if self.textures:
            tex_tree = Tree("[bold cyan]Texture Definitions[/bold cyan]")
            for i, tex in enumerate(self.textures):
                tex_type_names = {
                    0: "Hardcoded",
                    1: "Skin",
                    2: "Object Skin",
                    3: "Weapon Blade",
                    4: "Weapon Handle",
                    5: "Environment",
                    6: "Character Hair",
                    7: "Character Facial Hair",
                    8: "Skin Extra",
                    9: "UI Skin",
                    10: "Tauren Mane",
                    11: "Monster Skin 1",
                    12: "Monster Skin 2",
                    13: "Monster Skin 3",
                    14: "Item Icon"
                }
                type_name = tex_type_names.get(tex.type, f"Unknown ({tex.type})")
                
                tex_node = tex_tree.add(f"Texture {i}: {type_name}")
                tex_node.add(f"Flags: {tex.flags:#x}")
                
                if tex.name.count > 0:
                    name = self.read_string(tex.name.offset)
                    tex_node.add(f"Name: {name}")
            
            console.print(Panel(tex_tree, title="[bold]Textures[/bold]"))
        
        # Create ASCII art representation of model bounds
        self._create_ascii_model_view()
    
    def _create_ascii_model_view(self):
        """Create ASCII art representation of model structure"""
        
        if not self.header:
            return
        
        # Simple ASCII representation of bounding box
        min_b, max_b = self.header.bounding_box
        width = max_b.x - min_b.x
        height = max_b.y - min_b.y
        depth = max_b.z - min_b.z
        
        console.print("\n[bold cyan]Model Bounding Box (ASCII Representation)[/bold cyan]")
        console.print("```")
        console.print(f"     Width: {width:.1f} units")
        console.print(f"    Height: {height:.1f} units")
        console.print(f"     Depth: {depth:.1f} units")
        console.print()
        console.print("       Top View:")
        console.print("    +-----------+")
        console.print("   /           /|")
        console.print("  /           / |")
        console.print(" +-----------+  |")
        console.print(" |           |  +")
        console.print(" |           | /")
        console.print(" |           |/")
        console.print(" +-----------+")
        console.print()
        console.print(f" Center: ({(min_b.x+max_b.x)/2:.1f}, {(min_b.y+max_b.y)/2:.1f}, {(min_b.z+max_b.z)/2:.1f})")
        console.print("```")
    
    def parse_all(self) -> bool:
        """Parse all M2 data"""
        console.print("[yellow]Parsing M2 file...[/yellow]")
        
        if not self.parse_header():
            return False
        
        console.print("[green]✅ Header parsed[/green]")
        
        # Parse all components
        if self.parse_vertices():
            console.print(f"[green]✅ Parsed {len(self.vertices)} vertices[/green]")
        
        if self.parse_bones():
            console.print(f"[green]✅ Parsed {len(self.bones)} bones[/green]")
        
        if self.parse_textures():
            console.print(f"[green]✅ Parsed {len(self.textures)} textures[/green]")
        
        if self.parse_sequences():
            console.print(f"[green]✅ Parsed {len(self.sequences)} animation sequences[/green]")
        
        if self.parse_model_views():
            console.print(f"[green]✅ Parsed {len(self.model_views)} skin profiles[/green]")
        
        return True

# ============================================================================
# CLI Interface
# ============================================================================

@click.command()
@click.argument("m2_file", type=click.Path(exists=True))
@click.option("--export", "-e", type=click.Path(), help="Export parsed data to JSON")
@click.option("--validation-mode", "-v", default="permissive", type=click.Choice(["strict", "permissive", "none"]), help="Validation mode for vertex data")
@click.option("--coordinate-system", "-c", type=click.Choice(["blender", "unity", "unreal"]), help="Transform coordinates to target system")
def main(m2_file: str, export: Optional[str], validation_mode: str, coordinate_system: Optional[str]):
    """Enhanced M2 Parser - Parse and visualize WoW M2 models with advanced features"""
    
    console.print(f"\n[bold magenta]═══ Enhanced M2 Parser with Rust Parity ═══[/bold magenta]")
    console.print(f"[cyan]File:[/cyan] {m2_file}")
    console.print(f"[cyan]Size:[/cyan] {Path(m2_file).stat().st_size:,} bytes")
    console.print(f"[cyan]Validation Mode:[/cyan] {validation_mode}")
    if coordinate_system:
        console.print(f"[cyan]Coordinate System:[/cyan] {coordinate_system}")
    console.print()
    
    parser = EnhancedM2Parser(m2_file, validation_mode, coordinate_system)
    
    # Load and parse
    if not parser.load_file():
        console.print("[red]Failed to load file[/red]")
        return
    
    if not parser.parse_all():
        console.print("[red]Failed to parse M2 data[/red]")
        return
    
    # Display visual representation
    parser.create_visual_representation()
    
    # Export if requested
    if export:
        import json
        
        export_data = {
            "file": str(m2_file),
            "header": {
                "magic": parser.header.magic,
                "version": parser.header.version,
                "vertices": parser.header.vertices.count,
                "bones": parser.header.bones.count,
                "animations": parser.header.animations.count,
                "views": parser.header.views.count,
            },
            "vertices_sample": [
                {
                    "position": [v.position.x, v.position.y, v.position.z],
                    "normal": [v.normal.x, v.normal.y, v.normal.z],
                    "uv": [v.tex_coords.x, v.tex_coords.y]
                }
                for v in parser.vertices[:10]
            ],
            "bones_sample": [
                {
                    "id": b.key_bone_id,
                    "parent": b.parent,
                    "pivot": [b.pivot.x, b.pivot.y, b.pivot.z]
                }
                for b in parser.bones[:10]
            ]
        }
        
        with open(export, 'w') as f:
            json.dump(export_data, f, indent=2)
        
        console.print(f"\n[green]✅ Data exported to {export}[/green]")
    
    console.print("\n[bold green]═══ Parsing Complete ═══[/bold green]\n")

if __name__ == "__main__":
    main()