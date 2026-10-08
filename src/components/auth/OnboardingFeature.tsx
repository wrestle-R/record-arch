import { Button, Description, Modal, Surface } from "@heroui/react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import {
	ArrowLeft,
	ArrowRight,
	Check,
	Link,
	Monitor,
	Play,
	Record,
	SlidersHorizontal,
} from "@phosphor-icons/react";

const features = [
	{
		name: "Record",
		title: "Start with a moment.",
		description: "Choose your screen or a window. Add your voice and camera, then hit record.",
	},
	{
		name: "Preview",
		title: "Make it yours.",
		description: "Trim the take, frame your video, and bring the important details into focus.",
	},
	{
		name: "Share",
		title: "Ready when you are.",
		description:
			"Publish a link for anyone to watch, or save the finished video to your device.",
	},
];

// Decorative product illustrations stay crisp as the image frame changes size.
function FeatureIllustration({ step }: { step: number }) {
	return (
		<div
			aria-hidden="true"
			className="flex h-full items-center justify-center p-6 sm:p-10"
		>
			<div className="relative w-full max-w-[580px] overflow-hidden rounded-2xl border border-white/50 bg-white/85 text-zinc-900 shadow-2xl backdrop-blur-xl">
				<div className="flex items-center gap-1.5 border-b border-black/5 px-4 py-3">
					{[0, 1, 2].map((i) => (
						<span key={i} className="size-2 rounded-full bg-zinc-300" />
					))}
					<span className="ml-3 text-xs font-medium">
						{features[step].name === "Share" ? "Your recording" : "Recordly"}
					</span>
				</div>
				<div className="relative mx-5 mb-5 mt-4 flex aspect-video items-center justify-center overflow-hidden rounded-lg bg-gradient-to-br from-indigo-200 via-sky-100 to-rose-200">
					<div className="flex w-3/5 flex-col gap-3 rounded-lg bg-white/90 p-5 shadow-lg">
						<div className="h-2 w-1/3 rounded bg-indigo-300" />
						<div className="h-3 w-4/5 rounded bg-zinc-700" />
						<div className="h-2 w-full rounded bg-zinc-200" />
						<div className="h-2 w-2/3 rounded bg-zinc-200" />
					</div>
					{step === 0 && (
						<div className="absolute bottom-3 flex items-center gap-3 rounded-xl bg-white px-4 py-2 shadow-lg">
							<Monitor size={20} />
							<span className="text-xs">Entire screen</span>
							<Record size={24} weight="fill" className="text-red-500" />
						</div>
					)}
					{step === 1 && (
						<div className="absolute bottom-3 flex items-center gap-3 rounded-xl bg-white px-4 py-2 shadow-lg">
							<SlidersHorizontal size={20} />
							<div className="h-1.5 w-24 rounded-full bg-blue-500" />
							<span className="text-xs">Preview</span>
						</div>
					)}
					{step === 2 && (
						<div className="absolute flex size-14 items-center justify-center rounded-full bg-white/90 shadow-lg">
							<Play size={24} weight="fill" />
						</div>
					)}
				</div>
				{step === 2 && (
					<div className="mx-5 mb-5 flex items-center justify-center gap-2 rounded-lg bg-blue-500 p-3 text-sm font-medium text-white">
						<Link size={18} />
						Copy link
						<Check size={16} />
					</div>
				)}
			</div>
		</div>
	);
}

export function OnboardingFeature({
	step,
	onStep,
	onFinish,
}: {
	step: number;
	onStep: (step: number) => void;
	onFinish: () => void;
}) {
	const reduceMotion = useReducedMotion();
	const feature = features[step];
	return (
		<div className="absolute inset-0 flex flex-col">
			<div className="relative min-h-0 flex-1 overflow-hidden">
				<AnimatePresence mode="wait" initial={false}>
					<motion.div
						key={step}
						initial={{ opacity: 0, y: reduceMotion ? 0 : 12 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0 }}
						transition={{ duration: reduceMotion ? 0 : 0.2 }}
						className="absolute inset-0"
					>
						<FeatureIllustration step={step} />
					</motion.div>
				</AnimatePresence>
			</div>
			<Surface className="shrink-0 border-t border-separator px-6 py-5 sm:px-8 sm:py-6">
				<div className="flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between">
					<Modal.Header aria-live="polite" className="min-w-0 flex-1 gap-2">
						<Modal.Heading className="text-2xl font-semibold tracking-tight sm:text-3xl">
							{feature.title}
						</Modal.Heading>
						<Description className="text-sm leading-relaxed">
							{feature.description}
						</Description>
					</Modal.Header>
					<Modal.Footer className="shrink-0 gap-2">
						<Button
							isIconOnly
							aria-label={step === 0 ? "Back to sign in" : "Previous feature"}
							variant="secondary"
							onPress={() => onStep(step - 1)}
						>
							<ArrowLeft size={18} />
						</Button>
						<Button
							isIconOnly
							aria-label={step === 2 ? "Get started" : "Next"}
							variant="secondary"
							onPress={() => (step === 2 ? onFinish() : onStep(step + 1))}
						>
							<ArrowRight size={18} />
						</Button>
					</Modal.Footer>
				</div>
			</Surface>
		</div>
	);
}
