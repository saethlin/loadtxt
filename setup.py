from setuptools import setup


def build_native(spec):
    build = spec.add_external_build(
        cmd=['cargo', 'build', '--release'],
        path='./rust'
    )

    spec.add_cffi_module(
        module_path='loadtxt._native',
        dylib=lambda: build.find_dylib(
            'loadtxt',
            in_path='target/release'),
        header_filename=lambda: build.find_header(
            'loadtxt.h'),
        rtld_flags=[
            'NOW',
            'NODELETE'],
    )


setup(
    name='loadtxt',
    version='0.0.0',
    packages=['loadtxt'],
    zip_safe=False,
    platforms='any',
    setup_requires=['milksnake'],
    install_requires=['milksnake', 'numpy'],
    milksnake_tasks=[build_native],
)
