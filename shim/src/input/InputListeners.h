#pragma once

namespace levi_freecam {

// forwards key and mouse events to Rust and cancels the ones Rust consumes
class InputListeners {
public:
    static void registerListeners();
    static void unregisterListeners();
};

} // namespace levi_freecam
