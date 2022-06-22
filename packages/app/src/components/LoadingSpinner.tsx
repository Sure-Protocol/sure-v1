import { useIsLoading } from '../context/loadingProvider';

const LoadingSpinner: React.FunctionComponent<{ children?: JSX.Element }> = ({
	children,
}) => {
	const [isLoading] = useIsLoading();
	return <div>{isLoading ? <p>Loading...</p> : children}</div>;
};

export default LoadingSpinner;
