import type { Transition } from "motion/react";

/**
 * The official app's motion vocabulary (NotchMotion.swift), as Motion springs. SwiftUI's
 * `spring(response, dampingFraction)` maps exactly onto a mass-1 spring:
 * stiffness = (2π / response)², damping = 4π · dampingFraction / response.
 */
function swiftSpring(response: number, dampingFraction: number): Transition {
  return {
    type: "spring",
    mass: 1,
    stiffness: (2 * Math.PI / response) ** 2,
    damping: (4 * Math.PI * dampingFraction) / response,
  };
}

export const notchMotion = {
  /** Folding open and shut */
  unfold: swiftSpring(0.42, 0.78),
  /** Contents arriving after the shape has started opening */
  contents: swiftSpring(0.36, 0.82),
  /** The card travelling between cells: a bigger object moving further */
  glide: swiftSpring(0.5, 0.86),
  /** Contents changing inside something already moving: an ease, nothing to overshoot */
  crossfade: { duration: 0.16, ease: "easeInOut" } as Transition,
  /** A percentage changing: slow on purpose, a sweep reads as a measurement, a snap as a glitch */
  reading: swiftSpring(0.9, 0.9),
  /** The ring pressed in while a refresh is in flight */
  press: swiftSpring(0.3, 0.62),
  /** One full turn of the reading on refresh, easing out so it settles */
  refreshTurn: { duration: 0.95, ease: [0.32, 0, 0.14, 1] } as Transition,
} as const;
