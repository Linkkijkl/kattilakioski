import {
    resolve
} from 'path'
import {
    defineConfig
} from 'vite'

export default defineConfig({
    // https://vite.dev/config/shared-options.html
    publicDir: false,
    build: {
        rollupOptions: {
            input: {
                main: resolve(__dirname, 'index.html'),
            },
        },
    },
})
