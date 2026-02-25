import cudaq
from cudaq.dynamics import Schedule
from cudaq.operators import RydbergHamiltonian, ScalarOperator


def main():
    # Route through unified Pasqal target in QRMI mode.
    cudaq.set_target("pasqal", machine="qrmi")

    a = 5e-6
    register = [(a, 0.0), (2.0 * a, 0.0), (3.0 * a, 0.0)]
    time_ramp = 1.0e-6
    time_max = 3.0e-6
    steps = [0.0, time_ramp, time_max - time_ramp, time_max]
    schedule = Schedule(steps, ["t"])

    omega_max = 1.0e6
    delta_start = 0.0
    delta_end = 1.0e6
    omega = ScalarOperator(lambda t: omega_max if time_ramp < t.real < time_max else 0.0)
    phi = ScalarOperator.const(0.0)
    delta = ScalarOperator(lambda t: delta_end if time_ramp < t.real < time_max else delta_start)

    # Wait for cloud completion and print counts in both repr and dump formats.
    result = cudaq.evolve_async(
        RydbergHamiltonian(atom_sites=register,
                           amplitude=omega,
                           phase=phi,
                           delta_global=delta),
        schedule=schedule,
        shots_count=100,
    ).get()
    print(result)
    result.dump()


if __name__ == "__main__":
    main()
