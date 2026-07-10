/** @type {import('tailwindcss').Config} */
export default {
	content: [
		"./app/**/*.{js,ts,jsx,tsx,mdx}",
		"./components/**/*.{js,ts,jsx,tsx,mdx}",
	],
	safelist: [
		{
			// Keeps all standard colors compiled
			pattern:
				/^bg-(slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(400|500|600)$/,
		},
		"bg-white",
		"bg-black",
	],
	theme: {
		extend: {},
	},
	plugins: [],
};
