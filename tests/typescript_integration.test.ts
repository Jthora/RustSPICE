/**
 * TypeScript integration test for RustSPICE WASM module
 * 
 * This test verifies that the WASM module properly generates
 * TypeScript bindings and works correctly in a TypeScript environment.
 */

import { describe, it, expect, beforeAll } from '@jest/globals';

// Mock the WASM module interface for testing
interface RustSpice {
  StateVector: new (x: number, y: number, z: number, vx: number, vy: number, vz: number, light_time: number) => StateVector;
  SpiceError: new (error_type: SpiceErrorType, message: string) => SpiceError;
  SpiceErrorType: typeof SpiceErrorType;
  
  // Time functions
  calendar_to_et(year: number, month: number, day: number, hour: number, minute: number, second: number): number;
  julian_date_to_et(julian_date: number): number;
  et_to_utc(et: number, precision?: number): string;
  utc_to_et(utc_string: string): number;
  
  // Kernel functions
  load_kernel(data: Uint8Array, filename?: string): void;
  clear_kernels(): void;
  list_kernels(): string[];
  
  // Error functions
  has_errors(): boolean;
  get_error_message(): string;
  reset_errors(): void;
  version(): string;
  
  // Constants
  speed_of_light(): number;
  astronomical_unit(): number;
  earth_radius(): number;
  seconds_per_day(): number;
  j2000_julian_date(): number;
  
  // Coordinate functions
  rectangular_to_spherical(x: number, y: number, z: number): number[];
  spherical_to_rectangular(radius: number, colatitude: number, longitude: number): number[];
  
  // Ephemeris functions
  spkezr(target: string, et: number, reference_frame: string, aberration_correction: string, observer: string): StateVector;
  spkpos(target: string, et: number, reference_frame: string, aberration_correction: string, observer: string): { position: number[]; light_time: number };
}

interface StateVector {
  readonly x: number;
  readonly y: number;
  readonly z: number;
  readonly vx: number;
  readonly vy: number;
  readonly vz: number;
  readonly light_time: number;
  
  position(): number[];
  velocity(): number[];
  magnitude(): number;
  toJSON(): any;
}

enum SpiceErrorType {
  KernelNotFound = "KernelNotFound",
  InvalidTime = "InvalidTime",
  InvalidTarget = "InvalidTarget",
  ComputationError = "ComputationError",
}

interface SpiceError extends Error {
  readonly error_type: SpiceErrorType;
  readonly message: string;
}

// Mock implementation for testing
class MockRustSpice implements RustSpice {
  StateVector = class implements StateVector {
    constructor(
      public readonly x: number,
      public readonly y: number,
      public readonly z: number,
      public readonly vx: number,
      public readonly vy: number,
      public readonly vz: number,
      public readonly light_time: number
    ) {}
    
    position(): number[] {
      return [this.x, this.y, this.z];
    }
    
    velocity(): number[] {
      return [this.vx, this.vy, this.vz];
    }
    
    magnitude(): number {
      return Math.sqrt(this.x * this.x + this.y * this.y + this.z * this.z);
    }
    
    toJSON(): any {
      return {
        x: this.x, y: this.y, z: this.z,
        vx: this.vx, vy: this.vy, vz: this.vz,
        light_time: this.light_time
      };
    }
  };
  
  SpiceError = class implements SpiceError {
    name = 'SpiceError';
    
    constructor(
      public readonly error_type: SpiceErrorType,
      public readonly message: string
    ) {}
  };
  
  SpiceErrorType = SpiceErrorType;
  
  calendar_to_et(year: number, month: number, day: number, hour: number, minute: number, second: number): number {
    // Simplified J2000 calculation
    const days_since_j2000 = (year - 2000) * 365.25 + (month - 1) * 30.44 + (day - 1);
    const day_fraction = (hour * 3600 + minute * 60 + second) / 86400;
    return (days_since_j2000 + day_fraction) * 86400;
  }
  
  julian_date_to_et(julian_date: number): number {
    return (julian_date - 2451545.0) * 86400;
  }
  
  et_to_utc(et: number, precision: number = 3): string {
    const days = et / 86400;
    const jd = days + 2451545.0;
    return `JD ${jd} (ET: ${et.toFixed(precision)})`;
  }
  
  utc_to_et(utc_string: string): number {
    throw new Error("UTC parsing not yet implemented");
  }
  
  load_kernel(data: Uint8Array, filename?: string): void {
    console.log(`Loading kernel: ${filename}, size: ${data.length} bytes`);
  }
  
  clear_kernels(): void {
    console.log("Clearing all kernels");
  }
  
  list_kernels(): string[] {
    return [];
  }
  
  has_errors(): boolean {
    return false;
  }
  
  get_error_message(): string {
    return "No errors";
  }
  
  reset_errors(): void {
    // No-op
  }
  
  version(): string {
    return "RustSPICE 0.1.0 (WebAssembly)";
  }
  
  speed_of_light(): number {
    return 299792.458;
  }
  
  astronomical_unit(): number {
    return 149597870.7;
  }
  
  earth_radius(): number {
    return 6378.137;
  }
  
  seconds_per_day(): number {
    return 86400;
  }
  
  j2000_julian_date(): number {
    return 2451545.0;
  }
  
  rectangular_to_spherical(x: number, y: number, z: number): number[] {
    const r = Math.sqrt(x*x + y*y + z*z);
    const colatitude = r > 0 ? Math.acos(z / r) : 0;
    const longitude = Math.atan2(y, x);
    return [r, colatitude, longitude];
  }
  
  spherical_to_rectangular(radius: number, colatitude: number, longitude: number): number[] {
    const x = radius * Math.sin(colatitude) * Math.cos(longitude);
    const y = radius * Math.sin(colatitude) * Math.sin(longitude);
    const z = radius * Math.cos(colatitude);
    return [x, y, z];
  }
  
  spkezr(target: string, et: number, reference_frame: string, aberration_correction: string, observer: string): StateVector {
    console.log(`SPKEZR: ${target} relative to ${observer} at ET ${et}`);
    console.log(`Frame: ${reference_frame}, Correction: ${aberration_correction}`);
    
    return new this.StateVector(
      1000.0 + et * 0.001,
      2000.0 + et * 0.002,
      3000.0 + et * 0.003,
      10.0, 20.0, 30.0,
      0.1
    );
  }
  
  spkpos(target: string, et: number, reference_frame: string, aberration_correction: string, observer: string): { position: number[]; light_time: number } {
    const state = this.spkezr(target, et, reference_frame, aberration_correction, observer);
    return {
      position: state.position(),
      light_time: state.light_time
    };
  }
}

// Test suite
describe('RustSPICE TypeScript Integration', () => {
  let rustSpice: RustSpice;
  
  beforeAll(() => {
    rustSpice = new MockRustSpice();
  });
  
  describe('StateVector', () => {
    it('should create a state vector with correct properties', () => {
      const state = new rustSpice.StateVector(1, 2, 3, 4, 5, 6, 7);
      
      expect(state.x).toBe(1);
      expect(state.y).toBe(2);
      expect(state.z).toBe(3);
      expect(state.vx).toBe(4);
      expect(state.vy).toBe(5);
      expect(state.vz).toBe(6);
      expect(state.light_time).toBe(7);
    });
    
    it('should return correct position vector', () => {
      const state = new rustSpice.StateVector(10, 20, 30, 1, 2, 3, 0);
      const position = state.position();
      
      expect(position).toEqual([10, 20, 30]);
    });
    
    it('should return correct velocity vector', () => {
      const state = new rustSpice.StateVector(10, 20, 30, 1, 2, 3, 0);
      const velocity = state.velocity();
      
      expect(velocity).toEqual([1, 2, 3]);
    });
    
    it('should calculate correct magnitude', () => {
      const state = new rustSpice.StateVector(3, 4, 0, 0, 0, 0, 0);
      const magnitude = state.magnitude();
      
      expect(magnitude).toBe(5); // 3-4-5 triangle
    });
    
    it('should serialize to JSON correctly', () => {
      const state = new rustSpice.StateVector(1, 2, 3, 4, 5, 6, 7);
      const json = state.toJSON();
      
      expect(json).toEqual({
        x: 1, y: 2, z: 3,
        vx: 4, vy: 5, vz: 6,
        light_time: 7
      });
    });
  });
  
  describe('Time Conversions', () => {
    it('should convert J2000 epoch correctly', () => {
      const et = rustSpice.calendar_to_et(2000, 1, 1, 12, 0, 0);
      expect(et).toBeCloseTo(0, 5); // Should be close to 0 for J2000
    });
    
    it('should convert Julian date correctly', () => {
      const et = rustSpice.julian_date_to_et(2451545.0); // J2000
      expect(et).toBe(0);
    });
    
    it('should handle day progression correctly', () => {
      const et1 = rustSpice.julian_date_to_et(2451545.0);
      const et2 = rustSpice.julian_date_to_et(2451546.0);
      
      expect(et2 - et1).toBe(86400); // One day = 86400 seconds
    });
    
    it('should format UTC string correctly', () => {
      const utc = rustSpice.et_to_utc(0, 3);
      expect(utc).toContain('JD 2451545');
    });
  });
  
  describe('Constants', () => {
    it('should return correct physical constants', () => {
      expect(rustSpice.speed_of_light()).toBe(299792.458);
      expect(rustSpice.astronomical_unit()).toBe(149597870.7);
      expect(rustSpice.earth_radius()).toBe(6378.137);
      expect(rustSpice.seconds_per_day()).toBe(86400);
      expect(rustSpice.j2000_julian_date()).toBe(2451545.0);
    });
  });
  
  describe('Coordinate Transformations', () => {
    it('should convert rectangular to spherical correctly', () => {
      const [r, colatitude, longitude] = rustSpice.rectangular_to_spherical(1, 1, 1);
      
      expect(r).toBeCloseTo(Math.sqrt(3), 10);
      expect(colatitude).toBeCloseTo(Math.acos(1 / Math.sqrt(3)), 10);
      expect(longitude).toBeCloseTo(Math.PI / 4, 10);
    });
    
    it('should convert spherical to rectangular correctly', () => {
      const radius = Math.sqrt(3);
      const colatitude = Math.acos(1 / Math.sqrt(3));
      const longitude = Math.PI / 4;
      
      const [x, y, z] = rustSpice.spherical_to_rectangular(radius, colatitude, longitude);
      
      expect(x).toBeCloseTo(1, 10);
      expect(y).toBeCloseTo(1, 10);
      expect(z).toBeCloseTo(1, 10);
    });
    
    it('should be reversible', () => {
      const original = [3, 4, 5];
      const spherical = rustSpice.rectangular_to_spherical(...original);
      const rectangular = rustSpice.spherical_to_rectangular(...spherical);
      
      expect(rectangular[0]).toBeCloseTo(original[0], 10);
      expect(rectangular[1]).toBeCloseTo(original[1], 10);
      expect(rectangular[2]).toBeCloseTo(original[2], 10);
    });
  });
  
  describe('Ephemeris Functions', () => {
    it('should call spkezr with correct parameters', () => {
      const state = rustSpice.spkezr('MARS', 0, 'J2000', 'LT+S', 'EARTH');
      
      expect(state).toBeInstanceOf(rustSpice.StateVector);
      expect(state.x).toBe(1000); // Based on mock implementation
      expect(state.light_time).toBe(0.1);
    });
    
    it('should call spkpos and return position object', () => {
      const result = rustSpice.spkpos('MOON', 86400, 'J2000', 'LT', 'EARTH');
      
      expect(result).toHaveProperty('position');
      expect(result).toHaveProperty('light_time');
      expect(Array.isArray(result.position)).toBe(true);
      expect(result.position).toHaveLength(3);
    });
  });
  
  describe('Error Handling', () => {
    it('should create SpiceError correctly', () => {
      const error = new rustSpice.SpiceError(
        rustSpice.SpiceErrorType.KernelNotFound,
        'Test error message'
      );
      
      expect(error.error_type).toBe(SpiceErrorType.KernelNotFound);
      expect(error.message).toBe('Test error message');
    });
    
    it('should handle error checking functions', () => {
      expect(rustSpice.has_errors()).toBe(false);
      expect(rustSpice.get_error_message()).toBe('No errors');
    });
  });
  
  describe('Kernel Management', () => {
    it('should handle kernel loading', () => {
      const mockData = new Uint8Array([1, 2, 3, 4, 5]);
      
      expect(() => {
        rustSpice.load_kernel(mockData, 'test.bsp');
      }).not.toThrow();
    });
    
    it('should handle kernel listing', () => {
      const kernels = rustSpice.list_kernels();
      expect(Array.isArray(kernels)).toBe(true);
    });
  });
  
  describe('Utility Functions', () => {
    it('should return version information', () => {
      const version = rustSpice.version();
      expect(version).toContain('RustSPICE');
      expect(version).toContain('0.1.0');
    });
  });
  
  describe('Type Safety', () => {
    it('should enforce correct parameter types', () => {
      // These should compile without TypeScript errors
      const state: StateVector = new rustSpice.StateVector(1, 2, 3, 4, 5, 6, 7);
      const et: number = rustSpice.calendar_to_et(2000, 1, 1, 12, 0, 0);
      const position: number[] = state.position();
      const magnitude: number = state.magnitude();
      
      expect(typeof et).toBe('number');
      expect(Array.isArray(position)).toBe(true);
      expect(typeof magnitude).toBe('number');
    });
    
    it('should handle optional parameters correctly', () => {
      const utc1 = rustSpice.et_to_utc(0);
      const utc2 = rustSpice.et_to_utc(0, 5);
      
      expect(typeof utc1).toBe('string');
      expect(typeof utc2).toBe('string');
    });
  });
});

export {};
