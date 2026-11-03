import { describe, expect, it } from "vitest";
import { wheelCount, wheelPositions } from "./odometer";

describe("wheelPositions", () => {
  it("shows whole numbers digit by digit", () => {
    expect(wheelPositions(51, 2)).toEqual([51, 5]);
    expect(wheelPositions(100, 3)).toEqual([100, 10, 1]);
  });

  it("keeps higher wheels still until the lower wheel rolls past 9", () => {
    expect(wheelPositions(14.5, 2)[1]).toBe(1);
    expect(wheelPositions(19.25, 2)[1]).toBeCloseTo(1.25, 10);
    expect(wheelPositions(19.75, 2)[1]).toBeCloseTo(1.75, 10);
    expect(wheelPositions(20, 2)[1]).toBe(2);
  });

  it("carries through several wheels at once", () => {
    const [, tens, hundreds] = wheelPositions(99.5, 3);
    expect(tens).toBeCloseTo(9.5, 10);
    expect(hundreds).toBeCloseTo(0.5, 10);
  });

  it("never goes below zero", () => {
    expect(wheelPositions(-3, 2)).toEqual([0, 0]);
  });
});

describe("wheelCount", () => {
  it("fits the largest value", () => {
    expect(wheelCount(7)).toBe(1);
    expect(wheelCount(51)).toBe(2);
    expect(wheelCount(100)).toBe(3);
  });
});
