import MainButton from './components/MainButton';
import { css } from '@emotion/css';
import { useForm } from 'react-hook-form';
export const ManageMarkets = () => {
	const {
		register,
		handleSubmit,
		formState: { errors },
	} = useForm();
	const onSubmit = (data) => console.log(data);
	return (
		<div className="action-container">
			<div className="action-container-inner">
				<form
					className={css`
						display: flex;
						flex-direction: column;
					`}
					onSubmit={handleSubmit(onSubmit)}
				>
					<p>Create new Sure market</p>
					<input {...register('protocolnNme')} placeholder="Protocol Name " />
					<input {...register('ticker')} placeholder="Ticker" />
					<input {...register('programId')} placeholder="Program Id " />
					<input {...register('token')} placeholder="Token" />
					<MainButton>
						<p className="p--white p--margin-0">Submit</p>
					</MainButton>
				</form>
			</div>
		</div>
	);
};
